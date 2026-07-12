use crate::SliceError;

use super::{
    PackagePath,
    bytes::{
        checked_offset, has_signature, read_u16, read_u32, read_u64, require_signature, slice,
    },
    central::{extra_fields, zip64_values},
    invalid_entry, invalid_entry_metadata,
};

const LOCAL_HEADER: &[u8; 4] = b"PK\x03\x04";
const DATA_DESCRIPTOR: &[u8; 4] = b"PK\x07\x08";

#[derive(Clone, Copy)]
pub(super) struct LocalExpectation {
    pub flags: u16,
    pub method: u16,
    pub crc32: u32,
    pub compressed_size: u64,
    pub size: u64,
    pub offset: u64,
    pub zip64_sizes: bool,
}

pub(super) fn validate_local(
    input: &[u8],
    raw_name: &[u8],
    path: &PackagePath,
    expected: LocalExpectation,
) -> Result<(), SliceError> {
    let offset = usize::try_from(expected.offset)
        .map_err(|_| invalid_entry(path, "has an oversized local-header offset"))?;
    let (flags, method, crc32, compressed32, size32, data_start, local_name, extra) = (|| {
        require_signature(input, offset, LOCAL_HEADER, "local header")?;
        let flags = read_u16(input, offset, 6)?;
        let method = read_u16(input, offset, 8)?;
        let crc32 = read_u32(input, offset, 14)?;
        let compressed32 = read_u32(input, offset, 18)?;
        let size32 = read_u32(input, offset, 22)?;
        let name_len = usize::from(read_u16(input, offset, 26)?);
        let extra_len = usize::from(read_u16(input, offset, 28)?);
        let name_start = checked_offset(offset, 30)?;
        let extra_start = checked_offset(name_start, name_len)?;
        let data_start = checked_offset(extra_start, extra_len)?;
        let local_name = slice(input, name_start, name_len)?;
        let extra = slice(input, extra_start, extra_len)?;
        Ok::<_, SliceError>((
            flags,
            method,
            crc32,
            compressed32,
            size32,
            data_start,
            local_name,
            extra,
        ))
    })()
    .map_err(|error| invalid_entry_metadata(path, "local-header metadata", error))?;
    if flags != expected.flags || method != expected.method || local_name != raw_name {
        return Err(invalid_entry(path, "has conflicting local-header metadata"));
    }

    if flags & (1 << 3) == 0 {
        let (size, compressed) = (|| {
            let fields = extra_fields(extra)?;
            let (size, compressed, _, _) = zip64_values(size32, compressed32, 0, fields.zip64)?;
            Ok::<_, SliceError>((size, compressed))
        })()
        .map_err(|error| invalid_entry_metadata(path, "local ZIP64 extra field", error))?;
        if crc32 != expected.crc32
            || size != expected.size
            || compressed != expected.compressed_size
        {
            return Err(invalid_entry(
                path,
                "has conflicting local size or CRC metadata",
            ));
        }
        return ensure_payload_range(input, data_start, expected.compressed_size, path);
    }

    if (crc32 != 0 && crc32 != expected.crc32)
        || (compressed32 != 0
            && compressed32 != u32::MAX
            && u64::from(compressed32) != expected.compressed_size)
        || (size32 != 0 && size32 != u32::MAX && u64::from(size32) != expected.size)
    {
        return Err(invalid_entry(
            path,
            "has conflicting descriptor placeholders",
        ));
    }
    ensure_payload_range(input, data_start, expected.compressed_size, path)?;
    validate_descriptor(input, data_start, path, expected)
}

fn validate_descriptor(
    input: &[u8],
    data_start: usize,
    path: &PackagePath,
    expected: LocalExpectation,
) -> Result<(), SliceError> {
    let cursor = checked_offset(
        data_start,
        usize::try_from(expected.compressed_size)
            .map_err(|_| invalid_entry(path, "has an oversized descriptor offset"))?,
    )
    .map_err(|_| invalid_entry(path, "has an overflowing descriptor offset"))?;
    let (crc32, compressed, size) = (|| {
        let mut cursor = cursor;
        if has_signature(input, cursor, DATA_DESCRIPTOR) {
            cursor = checked_offset(cursor, 4)?;
        }
        let crc32 = read_u32(input, cursor, 0)?;
        cursor = checked_offset(cursor, 4)?;
        let (compressed, size) = if expected.zip64_sizes {
            (read_u64(input, cursor, 0)?, read_u64(input, cursor, 8)?)
        } else {
            (
                u64::from(read_u32(input, cursor, 0)?),
                u64::from(read_u32(input, cursor, 4)?),
            )
        };
        Ok::<_, SliceError>((crc32, compressed, size))
    })()
    .map_err(|error| invalid_entry_metadata(path, "data descriptor", error))?;
    if crc32 != expected.crc32 || compressed != expected.compressed_size || size != expected.size {
        return Err(invalid_entry(path, "has a conflicting data descriptor"));
    }
    Ok(())
}

fn ensure_payload_range(
    input: &[u8],
    data_start: usize,
    compressed_size: u64,
    path: &PackagePath,
) -> Result<(), SliceError> {
    let compressed_size = usize::try_from(compressed_size)
        .map_err(|_| invalid_entry(path, "has an oversized compressed size"))?;
    if data_start
        .checked_add(compressed_size)
        .is_none_or(|end| end > input.len())
    {
        return Err(invalid_entry(
            path,
            "compressed payload exceeds the archive",
        ));
    }
    Ok(())
}
