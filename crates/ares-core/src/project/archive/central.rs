use std::collections::BTreeSet;

use crate::SliceError;

use super::{
    ArchiveLimits, PackagePath,
    bytes::{
        checked_offset, has_signature, read_u16, read_u32, read_u64, require_signature, slice,
    },
    invalid_archive, invalid_entry, invalid_entry_metadata,
    local::{LocalExpectation, validate_local},
};

const CENTRAL_HEADER: &[u8; 4] = b"PK\x01\x02";
const EOCD: &[u8; 4] = b"PK\x05\x06";
const ZIP64_EOCD: &[u8; 4] = b"PK\x06\x06";
const ZIP64_LOCATOR: &[u8; 4] = b"PK\x06\x07";

pub(super) struct EntryMetadata {
    pub path: PackagePath,
    pub compressed_size: u64,
    pub size: u64,
    pub crc32: u32,
    pub method: u16,
}

pub(super) fn preflight(
    input: &[u8],
    limits: ArchiveLimits,
) -> Result<Vec<EntryMetadata>, SliceError> {
    let directory = find_directory(input)?;
    if directory.count > limits.max_entries {
        return Err(invalid_archive(format!(
            "contains {} entries, above the {} entry limit",
            directory.count, limits.max_entries
        )));
    }

    let mut cursor = directory.start;
    let mut total_size = 0u64;
    let mut raw_names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut entries = Vec::with_capacity(directory.count);
    for index in 0..directory.count {
        require_signature(input, cursor, CENTRAL_HEADER, "central header")?;
        let flags = read_u16(input, cursor, 8)?;
        let method = read_u16(input, cursor, 10)?;
        let crc32 = read_u32(input, cursor, 16)?;
        let compressed32 = read_u32(input, cursor, 20)?;
        let size32 = read_u32(input, cursor, 24)?;
        let name_len = usize::from(read_u16(input, cursor, 28)?);
        let extra_len = usize::from(read_u16(input, cursor, 30)?);
        let comment_len = usize::from(read_u16(input, cursor, 32)?);
        if read_u16(input, cursor, 34)? != 0 {
            return Err(invalid_archive("multi-disk ZIP entries are unsupported"));
        }
        let offset32 = read_u32(input, cursor, 42)?;
        let name_start = checked_offset(cursor, 46)?;
        let extra_start = checked_offset(name_start, name_len)?;
        let comment_start = checked_offset(extra_start, extra_len)?;
        let end = checked_offset(comment_start, comment_len)?;
        let raw_name = slice(input, name_start, name_len)?;
        let extra = slice(input, extra_start, extra_len)?;
        if !raw_names.insert(raw_name) {
            return Err(invalid_archive(format!(
                "entry {index} repeats an exact raw name"
            )));
        }

        let path = PackagePath::entry(raw_name)
            .map_err(|_| invalid_archive(format!("entry {index} has an invalid package path")))?;
        let extra_fields = extra_fields(extra).map_err(|error| {
            invalid_entry_metadata(&path, "central-directory extra field", error)
        })?;
        validate_unicode_alias(index, raw_name, &path, extra_fields.unicode_path)?;
        if !paths.insert(path.clone()) {
            return Err(invalid_entry(&path, "duplicates a normalized package path"));
        }

        let (size, compressed_size, local_offset, zip64_sizes) =
            zip64_values(size32, compressed32, offset32, extra_fields.zip64).map_err(|error| {
                invalid_entry_metadata(&path, "central-directory ZIP64 field", error)
            })?;
        validate_limits(&path, size, compressed_size, limits, &mut total_size)?;
        if flags & 1 != 0 {
            return Err(invalid_entry(&path, "is encrypted"));
        }
        if !matches!(method, 0 | 8) {
            return Err(invalid_entry(&path, "uses unsupported compression"));
        }
        validate_local(
            input,
            raw_name,
            &path,
            LocalExpectation {
                flags,
                method,
                crc32,
                compressed_size,
                size,
                offset: local_offset,
                zip64_sizes,
            },
        )?;
        entries.push(EntryMetadata {
            path,
            compressed_size,
            size,
            crc32,
            method,
        });
        cursor = end;
    }
    if cursor != directory.end {
        return Err(invalid_archive(
            "central-directory size disagrees with its entries",
        ));
    }
    Ok(entries)
}

#[derive(Clone, Copy)]
struct Directory {
    start: usize,
    end: usize,
    count: usize,
}

fn find_directory(input: &[u8]) -> Result<Directory, SliceError> {
    if input.len() < 22 {
        return Err(invalid_archive(
            "is missing an end-of-central-directory record",
        ));
    }
    let search_start = input.len().saturating_sub(22 + usize::from(u16::MAX));
    let eocd = (search_start..=input.len() - 22)
        .rev()
        .find(|&offset| {
            has_signature(input, offset, EOCD)
                && read_u16(input, offset, 20).ok().and_then(|length| {
                    checked_offset(offset, 22)
                        .and_then(|end| checked_offset(end, usize::from(length)))
                        .ok()
                }) == Some(input.len())
        })
        .ok_or_else(|| invalid_archive("is missing a valid end record"))?;
    if read_u16(input, eocd, 4)? != 0 || read_u16(input, eocd, 6)? != 0 {
        return Err(invalid_archive("multi-disk ZIP archives are unsupported"));
    }
    let count_on_disk = read_u16(input, eocd, 8)?;
    let count = read_u16(input, eocd, 10)?;
    let size = read_u32(input, eocd, 12)?;
    let offset = read_u32(input, eocd, 16)?;
    if count_on_disk != count {
        return Err(invalid_archive("central-directory counts conflict"));
    }
    if count != u16::MAX && size != u32::MAX && offset != u32::MAX {
        return checked_directory(offset.into(), size.into(), count.into(), eocd);
    }
    find_zip64_directory(input, eocd)
}

fn find_zip64_directory(input: &[u8], eocd: usize) -> Result<Directory, SliceError> {
    let locator = eocd
        .checked_sub(20)
        .ok_or_else(|| invalid_archive("ZIP64 locator is missing"))?;
    require_signature(input, locator, ZIP64_LOCATOR, "ZIP64 locator")?;
    if read_u32(input, locator, 4)? != 0 || read_u32(input, locator, 16)? != 1 {
        return Err(invalid_archive("multi-disk ZIP64 archives are unsupported"));
    }
    let zip64 = usize::try_from(read_u64(input, locator, 8)?)
        .map_err(|_| invalid_archive("ZIP64 end offset is too large"))?;
    require_signature(input, zip64, ZIP64_EOCD, "ZIP64 end record")?;
    if read_u32(input, zip64, 16)? != 0 || read_u32(input, zip64, 20)? != 0 {
        return Err(invalid_archive("multi-disk ZIP64 archives are unsupported"));
    }
    let count_on_disk = read_u64(input, zip64, 24)?;
    let count = read_u64(input, zip64, 32)?;
    if count_on_disk != count {
        return Err(invalid_archive("ZIP64 central-directory counts conflict"));
    }
    checked_directory(
        read_u64(input, zip64, 48)?,
        read_u64(input, zip64, 40)?,
        count,
        zip64,
    )
}

fn checked_directory(
    offset: u64,
    size: u64,
    count: u64,
    end: usize,
) -> Result<Directory, SliceError> {
    let start =
        usize::try_from(offset).map_err(|_| invalid_archive("directory offset is too large"))?;
    let size = usize::try_from(size).map_err(|_| invalid_archive("directory size is too large"))?;
    let count = usize::try_from(count).map_err(|_| invalid_archive("entry count is too large"))?;
    if start.checked_add(size) != Some(end) {
        return Err(invalid_archive("central-directory range is inconsistent"));
    }
    Ok(Directory { start, end, count })
}

pub(super) struct ExtraFields<'a> {
    pub(super) zip64: Option<&'a [u8]>,
    unicode_path: Option<&'a [u8]>,
}

pub(super) fn extra_fields(mut extra: &[u8]) -> Result<ExtraFields<'_>, SliceError> {
    let mut result = ExtraFields {
        zip64: None,
        unicode_path: None,
    };
    while !extra.is_empty() {
        if extra.len() < 4 {
            return Err(invalid_archive("an extra field is truncated"));
        }
        let id = read_u16(extra, 0, 0)?;
        let len = usize::from(read_u16(extra, 0, 2)?);
        let end = checked_offset(4, len)?;
        if extra.len() < end {
            return Err(invalid_archive("an extra field length is invalid"));
        }
        let value = slice(extra, 4, len)?;
        let slot = match id {
            0x0001 => Some(&mut result.zip64),
            0x7075 => Some(&mut result.unicode_path),
            _ => None,
        };
        if let Some(slot) = slot
            && slot.replace(value).is_some()
        {
            return Err(invalid_archive("an extra field is repeated"));
        }
        extra = slice(extra, end, extra.len() - end)?;
    }
    Ok(result)
}

pub(super) fn zip64_values(
    size32: u32,
    compressed32: u32,
    offset32: u32,
    zip64: Option<&[u8]>,
) -> Result<(u64, u64, u64, bool), SliceError> {
    let mut zip64 = zip64.unwrap_or_default();
    let needs_size = size32 == u32::MAX;
    let needs_compressed = compressed32 == u32::MAX;
    let size = if needs_size {
        take_u64(&mut zip64)?
    } else {
        size32.into()
    };
    let compressed = if needs_compressed {
        take_u64(&mut zip64)?
    } else {
        compressed32.into()
    };
    let offset = if offset32 == u32::MAX {
        take_u64(&mut zip64)?
    } else {
        offset32.into()
    };
    Ok((size, compressed, offset, needs_size || needs_compressed))
}

fn validate_unicode_alias(
    index: usize,
    raw_name: &[u8],
    path: &PackagePath,
    unicode: Option<&[u8]>,
) -> Result<(), SliceError> {
    let Some(unicode) = unicode else {
        return Ok(());
    };
    if unicode.len() < 5 || unicode[0] != 1 || read_u32(unicode, 0, 1)? != crc32(raw_name) {
        return Err(invalid_archive(format!(
            "entry {index} has an invalid Unicode path alias"
        )));
    }
    let alias_bytes = slice(unicode, 5, unicode.len() - 5)?;
    let alias = PackagePath::entry(alias_bytes)
        .map_err(|_| invalid_archive(format!("entry {index} has an invalid Unicode path alias")))?;
    if &alias != path {
        return Err(invalid_archive(format!(
            "entry {index} has a conflicting Unicode path alias"
        )));
    }
    Ok(())
}

fn validate_limits(
    path: &PackagePath,
    size: u64,
    compressed: u64,
    limits: ArchiveLimits,
    total: &mut u64,
) -> Result<(), SliceError> {
    if size > limits.max_entry_size {
        return Err(invalid_entry(path, "exceeds the expanded entry-size limit"));
    }
    *total = total
        .checked_add(size)
        .ok_or_else(|| invalid_archive("expanded-size total overflows"))?;
    if *total > limits.max_total_size {
        return Err(invalid_entry(path, "exceeds the total expanded-size limit"));
    }
    if compressed == 0 && size != 0 {
        return Err(invalid_entry(
            path,
            "has zero compressed bytes for non-empty output",
        ));
    }
    if compressed != 0
        && u128::from(size) > u128::from(compressed) * u128::from(limits.max_expansion_ratio)
    {
        return Err(invalid_entry(path, "exceeds the expansion-ratio limit"));
    }
    Ok(())
}

fn take_u64(input: &mut &[u8]) -> Result<u64, SliceError> {
    let value = read_u64(input, 0, 0)?;
    *input = slice(input, 8, input.len() - 8)?;
    Ok(value)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
