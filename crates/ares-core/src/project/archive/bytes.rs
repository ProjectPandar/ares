use crate::SliceError;

use super::invalid_archive;

pub(super) fn checked_offset(base: usize, relative: usize) -> Result<usize, SliceError> {
    base.checked_add(relative)
        .ok_or_else(|| invalid_archive("metadata byte offset overflows"))
}

pub(super) fn slice(input: &[u8], offset: usize, len: usize) -> Result<&[u8], SliceError> {
    let end = checked_offset(offset, len)?;
    input
        .get(offset..end)
        .ok_or_else(|| invalid_archive("metadata byte range is truncated"))
}

pub(super) fn has_signature(input: &[u8], offset: usize, signature: &[u8; 4]) -> bool {
    slice(input, offset, signature.len()).ok() == Some(signature)
}

pub(super) fn require_signature(
    input: &[u8],
    offset: usize,
    signature: &[u8; 4],
    category: &str,
) -> Result<(), SliceError> {
    if !has_signature(input, offset, signature) {
        return Err(invalid_archive(format!("{category} signature is invalid")));
    }
    Ok(())
}

pub(super) fn read_u16(input: &[u8], base: usize, relative: usize) -> Result<u16, SliceError> {
    let offset = checked_offset(base, relative)?;
    Ok(u16::from_le_bytes(
        slice(input, offset, 2)?.try_into().unwrap(),
    ))
}

pub(super) fn read_u32(input: &[u8], base: usize, relative: usize) -> Result<u32, SliceError> {
    let offset = checked_offset(base, relative)?;
    Ok(u32::from_le_bytes(
        slice(input, offset, 4)?.try_into().unwrap(),
    ))
}

pub(super) fn read_u64(input: &[u8], base: usize, relative: usize) -> Result<u64, SliceError> {
    let offset = checked_offset(base, relative)?;
    Ok(u64::from_le_bytes(
        slice(input, offset, 8)?.try_into().unwrap(),
    ))
}
