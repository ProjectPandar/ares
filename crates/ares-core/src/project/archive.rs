mod bytes;
mod central;
mod limits;
mod local;
mod path;

use std::{collections::BTreeMap, io::Cursor, io::Read};

use zip::{CompressionMethod, ZipArchive};

use crate::SliceError;

use central::preflight;
pub(crate) use limits::ArchiveLimits;
pub(crate) use path::PackagePath;

pub(crate) struct ProjectArchive<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
    entries: BTreeMap<PackagePath, (usize, u64)>,
}

impl<'a> ProjectArchive<'a> {
    pub(crate) fn open(input: &'a [u8], limits: ArchiveLimits) -> Result<Self, SliceError> {
        let expected = preflight(input, limits)?;
        let mut archive = ZipArchive::new(Cursor::new(input))
            .map_err(|error| invalid_archive(format!("ZIP metadata is malformed: {error}")))?;
        if archive.len() != expected.len() {
            return Err(invalid_archive("central-directory entry count changed"));
        }

        let mut entries = BTreeMap::new();
        for (index, expected) in expected.into_iter().enumerate() {
            let file = archive.by_index_raw(index).map_err(|error| {
                invalid_archive(format!("entry {index} metadata is unreadable: {error}"))
            })?;
            let path = PackagePath::entry(file.name_raw()).map_err(|_| {
                invalid_archive(format!("entry {index} has an invalid package path"))
            })?;
            let method = match file.compression() {
                CompressionMethod::Stored => 0,
                CompressionMethod::Deflated => 8,
                _ => return Err(invalid_entry(&path, "uses unsupported compression")),
            };
            if path != expected.path
                || file.encrypted()
                || method != expected.method
                || file.compressed_size() != expected.compressed_size
                || file.size() != expected.size
                || file.crc32() != expected.crc32
            {
                return Err(invalid_entry(&path, "metadata changed during ZIP parsing"));
            }
            entries.insert(path, (index, expected.size));
        }
        Ok(Self { archive, entries })
    }

    pub(crate) fn read(&mut self, path: &PackagePath) -> Result<Vec<u8>, SliceError> {
        let &(index, declared_size) = self
            .entries
            .get(path)
            .ok_or_else(|| invalid_entry(path, "is missing"))?;
        let file = self
            .archive
            .by_index(index)
            .map_err(|error| invalid_entry(path, &format!("cannot be opened: {error}")))?;
        let limit = declared_size
            .checked_add(1)
            .ok_or_else(|| invalid_entry(path, "has an overflowing size"))?;
        let capacity = usize::try_from(declared_size)
            .map_err(|_| invalid_entry(path, "is too large for this platform"))?;
        let mut bytes = Vec::with_capacity(capacity);
        file.take(limit).read_to_end(&mut bytes).map_err(|error| {
            invalid_entry(path, &format!("failed CRC-checked expansion: {error}"))
        })?;
        if bytes.len() as u64 != declared_size {
            return Err(invalid_entry(path, "expanded size differs from metadata"));
        }
        Ok(bytes)
    }

    pub(crate) fn paths(&self) -> impl Iterator<Item = &PackagePath> {
        self.entries.keys()
    }
}

fn invalid_archive(reason: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(format!("invalid project archive: {}", reason.into()))
}

fn invalid_entry(path: &PackagePath, reason: &str) -> SliceError {
    invalid_archive(format!("entry {:?} {reason}", path.as_str()))
}

fn invalid_entry_metadata(path: &PackagePath, category: &str, error: SliceError) -> SliceError {
    invalid_entry(path, &format!("has invalid {category}: {error}"))
}
