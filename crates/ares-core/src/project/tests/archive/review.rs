use std::panic::catch_unwind;

use zip::CompressionMethod;

use super::{offsets, stream_zip, write_u16, write_u32, zip};
use crate::project::{ArchiveLimits, ProjectArchive};

const SECRET: &[u8] = b"SECRET-ARCHIVE-PAYLOAD";

#[test]
fn project_archive_returns_error_for_oversized_zip64_end_offset() {
    let mut bytes = zip(&[("entry.bin", b"x", CompressionMethod::Stored)]);
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == b"PK\x05\x06")
        .unwrap();
    write_u16(&mut bytes, eocd + 8, u16::MAX);
    write_u16(&mut bytes, eocd + 10, u16::MAX);
    write_u32(&mut bytes, eocd + 12, u32::MAX);
    write_u32(&mut bytes, eocd + 16, u32::MAX);

    let mut locator = b"PK\x06\x07".to_vec();
    locator.extend_from_slice(&0u32.to_le_bytes());
    locator.extend_from_slice(&u64::MAX.to_le_bytes());
    locator.extend_from_slice(&1u32.to_le_bytes());
    bytes.splice(eocd..eocd, locator);

    let outcome = catch_unwind(|| ProjectArchive::open(&bytes, ArchiveLimits::PROJECT));

    assert!(outcome.is_ok(), "hostile ZIP64 offset must not panic");
    assert!(outcome.unwrap().is_err());
}

#[test]
fn project_archive_returns_entry_error_for_oversized_zip64_local_offset() {
    let mut zip64_offset = b"\x01\0\x08\0".to_vec();
    zip64_offset.extend_from_slice(&u64::MAX.to_le_bytes());
    let bytes = raw_archive(
        b"%63ontext.bin",
        SECRET,
        (SECRET.len() as u32, SECRET.len() as u32, &[]),
        (u32::MAX, &zip64_offset),
    );

    let outcome = catch_unwind(|| ProjectArchive::open(&bytes, ArchiveLimits::PROJECT));

    assert!(outcome.is_ok(), "hostile local offset must not panic");
    let error = outcome.unwrap().err().expect("offset must be rejected");
    assert_entry_context(&error.to_string());
}

#[test]
fn project_archive_truncated_local_header_names_canonical_entry() {
    let mut bytes = zip(&[("%63ontext.bin", SECRET, CompressionMethod::Stored)]);
    let at = offsets(&bytes, 0);
    let truncated = u32::try_from(bytes.len() - 2).unwrap();
    write_u32(&mut bytes, at.central + 42, truncated);

    let error = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT)
        .err()
        .expect("truncated local header must be rejected")
        .to_string();

    assert_entry_context(&error);
}

#[test]
fn project_archive_truncated_local_zip64_extra_names_canonical_entry() {
    let malformed_zip64 = b"\x01\0\x08\0abc";
    let bytes = raw_archive(
        b"%63ontext.bin",
        SECRET,
        (u32::MAX, u32::MAX, malformed_zip64),
        (0, &[]),
    );

    let error = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT)
        .err()
        .expect("truncated local ZIP64 extra must be rejected")
        .to_string();

    assert_entry_context(&error);
}

#[test]
fn project_archive_truncated_descriptor_names_canonical_entry() {
    let mut bytes = stream_zip("%63ontext.bin", SECRET);
    let at = offsets(&bytes, 0);
    let compressed = u32::try_from(bytes.len() - at.data - 2).unwrap();
    write_u32(&mut bytes, at.central + 20, compressed);

    let error = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT)
        .err()
        .expect("truncated descriptor must be rejected")
        .to_string();

    assert_entry_context(&error);
}

fn assert_entry_context(error: &str) {
    assert!(error.contains("entry \"context.bin\""), "{error}");
    assert!(!error.contains("SECRET-ARCHIVE-PAYLOAD"), "{error}");
}

fn raw_archive(
    name: &[u8],
    payload: &[u8],
    local: (u32, u32, &[u8]),
    central: (u32, &[u8]),
) -> Vec<u8> {
    let (local_compressed, local_size, local_extra) = local;
    let (central_offset, central_extra) = central;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"PK\x03\x04");
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, local_compressed);
    push_u32(&mut bytes, local_size);
    push_u16(&mut bytes, name.len() as u16);
    push_u16(&mut bytes, local_extra.len() as u16);
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(local_extra);
    bytes.extend_from_slice(payload);

    let central_start = bytes.len();
    bytes.extend_from_slice(b"PK\x01\x02");
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, payload.len() as u32);
    push_u32(&mut bytes, payload.len() as u32);
    push_u16(&mut bytes, name.len() as u16);
    push_u16(&mut bytes, central_extra.len() as u16);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, central_offset);
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(central_extra);

    let central_size = bytes.len() - central_start;
    bytes.extend_from_slice(b"PK\x05\x06");
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 1);
    push_u32(&mut bytes, central_size as u32);
    push_u32(&mut bytes, central_start as u32);
    push_u16(&mut bytes, 0);
    bytes
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
