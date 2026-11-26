//! Lazy atomic `BufWriter<File>` implementation of `ares_core::GCodeSink`
//! (`GCode.cpp:2911-2965`).

use std::cell::RefCell;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use ares_core::{GCODE_OUTPUT_BLOCK_SIZE, GCodeSink, SliceError};

static NEXT_TMP_ID: AtomicU64 = AtomicU64::new(0);

enum SinkState {
    Unopened,
    Direct(BufWriter<File>),
    Staged {
        writer: BufWriter<File>,
        tmp_path: PathBuf,
    },
    Failed,
}

/// Streams finalized G-code chunks into a 64 KiB `BufWriter<File>`.
///
/// The destination file is opened lazily on the first non-empty chunk, so
/// validation, slicing, conflict, template, or printable-area errors never
/// create or overwrite the target path. Regular-file destinations stage to a
/// sibling `.tmp` file (`GCode.cpp:2911-2965`) and atomically rename on
/// [`Self::commit`]; dropping without committing removes the staged file.
pub(crate) struct AtomicFileSink {
    target: PathBuf,
    create_parent: bool,
    state: SinkState,
    error_slot: Option<Rc<RefCell<Option<io::Error>>>>,
    last_error: Option<io::Error>,
}

impl AtomicFileSink {
    pub(crate) fn new(target: PathBuf) -> Self {
        Self {
            target,
            create_parent: false,
            state: SinkState::Unopened,
            error_slot: None,
            last_error: None,
        }
    }

    pub(crate) fn in_directory(
        target: PathBuf,
        error_slot: Rc<RefCell<Option<io::Error>>>,
    ) -> Self {
        Self {
            target,
            create_parent: true,
            state: SinkState::Unopened,
            error_slot: Some(error_slot),
            last_error: None,
        }
    }

    pub(crate) fn take_error(&mut self) -> Option<io::Error> {
        self.last_error.take()
    }

    pub(crate) fn commit(mut self) -> io::Result<()> {
        match std::mem::replace(&mut self.state, SinkState::Failed) {
            SinkState::Unopened => {
                if self.create_parent
                    && let Some(parent) = self.target.parent()
                    && !parent.as_os_str().is_empty()
                {
                    fs::create_dir_all(parent)?;
                }
                File::create(&self.target)?;
                Ok(())
            }
            SinkState::Direct(mut writer) => writer.flush(),
            SinkState::Staged {
                mut writer,
                tmp_path,
            } => {
                let flushed = writer.flush();
                drop(writer);
                if let Err(error) = flushed {
                    let _ = fs::remove_file(&tmp_path);
                    return Err(error);
                }
                if let Err(error) = fs::rename(&tmp_path, &self.target) {
                    let _ = fs::remove_file(&tmp_path);
                    return Err(error);
                }
                Ok(())
            }
            SinkState::Failed => Err(self
                .last_error
                .take()
                .unwrap_or_else(|| io::Error::other("G-code sink write failed"))),
        }
    }

    fn open_if_needed(&mut self) -> io::Result<()> {
        if !matches!(self.state, SinkState::Unopened) {
            return Ok(());
        }
        if self.create_parent
            && let Some(parent) = self.target.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let direct = match fs::symlink_metadata(&self.target) {
            Ok(meta) => !meta.is_file(),
            Err(_) => false,
        };
        if direct {
            let file = File::create(&self.target)?;
            self.state = SinkState::Direct(BufWriter::with_capacity(GCODE_OUTPUT_BLOCK_SIZE, file));
            return Ok(());
        }
        let Some(tmp_path) = staging_path(&self.target) else {
            let file = File::create(&self.target)?;
            self.state = SinkState::Direct(BufWriter::with_capacity(GCODE_OUTPUT_BLOCK_SIZE, file));
            return Ok(());
        };
        let file = File::create(&tmp_path)?;
        self.state = SinkState::Staged {
            writer: BufWriter::with_capacity(GCODE_OUTPUT_BLOCK_SIZE, file),
            tmp_path,
        };
        Ok(())
    }

    fn record_error(&mut self, error: io::Error) -> SliceError {
        let message = format!("failed writing G-code output: {error}");
        self.cleanup();
        if let Some(slot) = &self.error_slot {
            *slot.borrow_mut() = Some(error);
        } else {
            self.last_error = Some(error);
        }
        SliceError::InvalidInput(message)
    }

    fn cleanup(&mut self) {
        if let SinkState::Staged { writer, tmp_path } =
            std::mem::replace(&mut self.state, SinkState::Failed)
        {
            drop(writer);
            let _ = fs::remove_file(tmp_path);
        }
    }
}

impl GCodeSink for AtomicFileSink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        if chunk.is_empty() {
            return Ok(());
        }
        if let Err(error) = self.open_if_needed() {
            return Err(self.record_error(error));
        }
        let written = match &mut self.state {
            SinkState::Direct(writer) | SinkState::Staged { writer, .. } => writer.write_all(chunk),
            SinkState::Unopened | SinkState::Failed => {
                Err(io::Error::other("G-code sink is closed"))
            }
        };
        if let Err(error) = written {
            return Err(self.record_error(error));
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SliceError> {
        let flushed = match &mut self.state {
            SinkState::Direct(writer) | SinkState::Staged { writer, .. } => writer.flush(),
            SinkState::Unopened | SinkState::Failed => Ok(()),
        };
        if let Err(error) = flushed {
            return Err(self.record_error(error));
        }
        Ok(())
    }
}

impl Drop for AtomicFileSink {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Plate output sink for multi-plate runs where `--outputdir` is optional.
pub(crate) enum PlateSink {
    File(AtomicFileSink),
    Discard,
}

impl PlateSink {
    pub(crate) fn commit(self) -> io::Result<()> {
        match self {
            Self::File(sink) => sink.commit(),
            Self::Discard => Ok(()),
        }
    }
}

impl GCodeSink for PlateSink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        match self {
            Self::File(sink) => sink.write_chunk(chunk),
            Self::Discard => Ok(()),
        }
    }

    fn flush(&mut self) -> Result<(), SliceError> {
        match self {
            Self::File(sink) => sink.flush(),
            Self::Discard => Ok(()),
        }
    }
}

fn staging_path(target: &Path) -> Option<PathBuf> {
    let file_name = target.file_name()?;
    let id = NEXT_TMP_ID.fetch_add(1, Ordering::Relaxed);
    let mut name = OsString::from(".");
    name.push(file_name);
    name.push(format!(".ares.{}.{id}.tmp", std::process::id()));
    Some(match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
        _ => PathBuf::from(name),
    })
}
