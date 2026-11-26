//! Streaming chunked G-code output sink (`GCode.hpp:411-437`,
//! `GCodeProcessor.hpp:603`, `GCodeProcessor.cpp:859-889`).

use std::io::Write;

use crate::SliceError;

/// Default streaming chunk size matching `GCodeProcessor::Output_Block_Size`
/// (`GCodeProcessor.hpp:603`).
pub const GCODE_OUTPUT_BLOCK_SIZE: usize = 65_536;

/// Platform-neutral sink receiving finalized G-code in contiguous chunks.
///
/// Mirrors the streaming boundary of upstream `GCode::GCodeOutputStream` and
/// `GCodeFileWriter`, without tying `ares-core` to filesystem I/O so native
/// CLI writers and browser WASM streams share one trait.
pub trait GCodeSink {
    /// Consume the next contiguous chunk of final G-code bytes.
    ///
    /// # Errors
    /// Returns a [`SliceError`] if the underlying destination rejects the write.
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError>;

    /// Flush any buffered bytes once emission completes.
    ///
    /// # Errors
    /// Returns a [`SliceError`] if flushing the destination fails.
    fn flush(&mut self) -> Result<(), SliceError> {
        Ok(())
    }

    /// Optional hint for the approximate total number of bytes to be emitted.
    fn reserve_hint(&mut self, _additional: usize) {}
}

impl<S: GCodeSink + ?Sized> GCodeSink for &mut S {
    #[inline]
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        (**self).write_chunk(chunk)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), SliceError> {
        (**self).flush()
    }

    #[inline]
    fn reserve_hint(&mut self, additional: usize) {
        (**self).reserve_hint(additional);
    }
}

impl GCodeSink for Vec<u8> {
    #[inline]
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        self.extend_from_slice(chunk);
        Ok(())
    }

    #[inline]
    fn reserve_hint(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

/// Adapter that forwards [`GCodeSink`] chunks to any [`std::io::Write`].
///
/// Preserves the last [`std::io::Error`] via [`Self::take_error`] so native
/// callers can inspect I/O failures without adding I/O variants to
/// [`SliceError`].
#[derive(Debug)]
pub struct IoWriteSink<W> {
    writer: W,
    bytes_written: usize,
    last_error: Option<std::io::Error>,
}

impl<W: Write> IoWriteSink<W> {
    #[must_use]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            bytes_written: 0,
            last_error: None,
        }
    }

    #[must_use]
    pub const fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    pub fn take_error(&mut self) -> Option<std::io::Error> {
        self.last_error.take()
    }

    #[must_use]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write> GCodeSink for IoWriteSink<W> {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        if chunk.is_empty() {
            return Ok(());
        }
        if let Err(error) = self.writer.write_all(chunk) {
            let message = format!("failed writing G-code chunk: {error}");
            self.last_error = Some(error);
            return Err(SliceError::InvalidInput(message));
        }
        self.bytes_written += chunk.len();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SliceError> {
        if let Err(error) = self.writer.flush() {
            let message = format!("failed flushing G-code sink: {error}");
            self.last_error = Some(error);
            return Err(SliceError::InvalidInput(message));
        }
        Ok(())
    }
}

/// Adapter that turns a fallible chunk callback into a [`GCodeSink`].
pub struct FnSink<F> {
    callback: F,
}

impl<F> FnSink<F>
where
    F: FnMut(&[u8]) -> Result<(), SliceError>,
{
    #[must_use]
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

/// Convenience constructor for a closure-backed [`GCodeSink`].
#[must_use]
pub fn fn_sink<F>(callback: F) -> FnSink<F>
where
    F: FnMut(&[u8]) -> Result<(), SliceError>,
{
    FnSink::new(callback)
}

impl<F> GCodeSink for FnSink<F>
where
    F: FnMut(&[u8]) -> Result<(), SliceError>,
{
    #[inline]
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        (self.callback)(chunk)
    }
}

/// Non-buffer outputs of a streaming slice pass (`ProcessedGCode` without the
/// monolithic `gcode: Vec<u8>` buffer).
#[derive(Clone, Debug, PartialEq)]
pub struct StreamedGCode {
    pub bytes_written: usize,
    /// PrintObject step warnings from the slice, not GCodeProcessor data.
    pub warnings: Vec<crate::PrintWarning>,
    pub result: crate::GCodeProcessorResult,
    /// `Print::print_statistics()` after export.
    pub print_statistics: crate::PrintStatistics,
    pub archive: Option<crate::SlicedArchive>,
}

impl StreamedGCode {
    pub(crate) fn into_processed(self, gcode: Vec<u8>) -> crate::ProcessedGCode {
        crate::ProcessedGCode {
            gcode,
            warnings: self.warnings,
            result: self.result,
            print_statistics: self.print_statistics,
            archive: self.archive,
        }
    }
}

/// Fixed-capacity block buffer mirroring `GCodeFileWriter` (`GCodeProcessor.cpp:859-889`).
pub(crate) struct ChunkWriter<'a> {
    sink: &'a mut dyn GCodeSink,
    buffer: Vec<u8>,
    bytes_written: usize,
}

impl<'a> ChunkWriter<'a> {
    pub(crate) fn new(sink: &'a mut dyn GCodeSink) -> Self {
        Self {
            sink,
            buffer: Vec::with_capacity(GCODE_OUTPUT_BLOCK_SIZE),
            bytes_written: 0,
        }
    }

    pub(crate) fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), SliceError> {
        if bytes.is_empty() {
            return Ok(());
        }
        if self.buffer.len() + bytes.len() > GCODE_OUTPUT_BLOCK_SIZE {
            self.flush_buffer()?;
            if bytes.len() >= GCODE_OUTPUT_BLOCK_SIZE {
                self.sink.write_chunk(bytes)?;
                self.bytes_written += bytes.len();
                return Ok(());
            }
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    pub(crate) fn flush_buffer(&mut self) -> Result<(), SliceError> {
        if !self.buffer.is_empty() {
            self.sink.write_chunk(&self.buffer)?;
            self.bytes_written += self.buffer.len();
            self.buffer.clear();
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<usize, SliceError> {
        self.flush_buffer()?;
        self.sink.flush()?;
        Ok(self.bytes_written)
    }
}
