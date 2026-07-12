use std::io::{Cursor, Write};

use zip::{CompressionMethod, ZipArchive as RawZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::project::{ArchiveLimits, PackagePath, ProjectArchive};

mod review;

#[test]
fn project_archive_limits_match_the_project_boundary() {
    assert_eq!(
        ArchiveLimits::PROJECT,
        ArchiveLimits {
            max_entries: 4_096,
            max_entry_size: 256 * 1024 * 1024,
            max_total_size: 1024 * 1024 * 1024,
            max_expansion_ratio: 1_000,
        }
    );
}

#[test]
fn project_archive_reads_stored_and_deflated_entries() {
    let bytes = zip(&[
        ("stored.txt", b"stored", CompressionMethod::Stored),
        ("3D/model.model", b"deflated", CompressionMethod::Deflated),
    ]);
    let mut archive = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).unwrap();

    assert_eq!(
        archive
            .read(&PackagePath::entry(b"stored.txt").unwrap())
            .unwrap(),
        b"stored"
    );
    assert_eq!(
        archive
            .read(&PackagePath::entry(b"3D/model.model").unwrap())
            .unwrap(),
        b"deflated"
    );
}

#[test]
fn project_archive_enforces_entry_count_limit() {
    assert!(ProjectArchive::open(&empty_zip(4_096), ArchiveLimits::PROJECT).is_ok());
    assert!(ProjectArchive::open(&empty_zip(4_097), ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_declared_entry_size_overflow() {
    let bytes = zip(&[("large.bin", b"four", CompressionMethod::Stored)]);

    assert!(ProjectArchive::open(&bytes, limits(8, 3, 8, 1_000)).is_err());
}

#[test]
fn project_archive_rejects_declared_total_size_overflow() {
    let bytes = zip(&[
        ("one.bin", b"one", CompressionMethod::Stored),
        ("two.bin", b"two", CompressionMethod::Stored),
    ]);

    assert!(ProjectArchive::open(&bytes, limits(8, 3, 5, 1_000)).is_err());
}

#[test]
fn project_archive_enforces_expansion_ratio_boundaries() {
    let mut exact = zip(&[("ratio.bin", b"x", CompressionMethod::Stored)]);
    patch_sizes(&mut exact, 0, 1, 1_000);
    assert!(ProjectArchive::open(&exact, limits(8, 2_000, 2_000, 1_000)).is_ok());

    let mut over = zip(&[("ratio.bin", b"x", CompressionMethod::Stored)]);
    patch_sizes(&mut over, 0, 1, 1_001);
    assert!(ProjectArchive::open(&over, limits(8, 2_000, 2_000, 1_000)).is_err());

    let mut zero_compressed = zip(&[("ratio.bin", b"x", CompressionMethod::Stored)]);
    patch_sizes(&mut zero_compressed, 0, 0, 1);
    assert!(ProjectArchive::open(&zero_compressed, limits(8, 2_000, 2_000, 1_000)).is_err());
}

#[test]
fn project_archive_rejects_encrypted_metadata() {
    let mut bytes = zip(&[("secret.bin", b"secret", CompressionMethod::Stored)]);
    let offsets = offsets(&bytes, 0);
    set_flag(&mut bytes, offsets.header + 6, 1);
    set_flag(&mut bytes, offsets.central + 8, 1);

    assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_unsupported_compression() {
    let mut bytes = zip(&[("method.bin", b"data", CompressionMethod::Stored)]);
    let offsets = offsets(&bytes, 0);
    write_u16(&mut bytes, offsets.header + 8, 12);
    write_u16(&mut bytes, offsets.central + 10, 12);

    assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_duplicate_normalized_paths() {
    for duplicate in ["/model.model", "%6dodel.model"] {
        let bytes = zip(&[
            ("model.model", b"first", CompressionMethod::Stored),
            (duplicate, b"second", CompressionMethod::Stored),
        ]);
        assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
    }
}

#[test]
fn project_archive_rejects_exact_duplicate_raw_names() {
    let mut bytes = zip(&[
        ("a", b"first", CompressionMethod::Stored),
        ("b", b"second", CompressionMethod::Stored),
    ]);
    let second = offsets(&bytes, 1);
    bytes[second.header + 30] = b'a';
    bytes[second.central + 46] = b'a';

    assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_unicode_path_alias_conflict() {
    let bytes = zip_with_unicode_alias("physical.model", "aliased.model");

    assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_local_header_conflicts() {
    assert_local_conflict(|bytes, at| write_u32(bytes, at.header + 22, 7));
    assert_local_conflict(|bytes, at| write_u32(bytes, at.header + 14, 7));
    assert_local_conflict(|bytes, at| write_u16(bytes, at.header + 8, 12));
    assert_local_conflict(|bytes, at| bytes[at.header + 30] = b'b');
    assert_local_conflict(|bytes, at| set_flag(bytes, at.header + 6, 1 << 11));
}

#[test]
fn project_archive_validates_data_descriptors() {
    let bytes = stream_zip("stream.bin", b"descriptor payload");
    let mut archive = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).unwrap();
    assert_eq!(
        archive
            .read(&PackagePath::entry(b"stream.bin").unwrap())
            .unwrap(),
        b"descriptor payload"
    );

    let mut corrupt = bytes;
    let at = offsets(&corrupt, 0);
    let descriptor = at.data + at.compressed_size as usize;
    assert_eq!(&corrupt[descriptor..descriptor + 4], b"PK\x07\x08");
    corrupt[descriptor + 4] ^= 1;
    assert!(ProjectArchive::open(&corrupt, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_validates_zip64_data_descriptors() {
    let bytes = stream_zip_with_options(
        "stream64.bin",
        b"zip64 descriptor payload",
        SimpleFileOptions::default().large_file(true),
    );
    let mut archive = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).unwrap();
    assert_eq!(
        archive
            .read(&PackagePath::entry(b"stream64.bin").unwrap())
            .unwrap(),
        b"zip64 descriptor payload"
    );

    let mut corrupt = bytes;
    let at = offsets(&corrupt, 0);
    let descriptor = at.data + at.compressed_size as usize;
    assert_eq!(&corrupt[descriptor..descriptor + 4], b"PK\x07\x08");
    corrupt[descriptor + 16] ^= 1;
    assert!(ProjectArchive::open(&corrupt, ArchiveLimits::PROJECT).is_err());
}

#[test]
fn project_archive_rejects_actual_entry_and_total_overflow() {
    let mut bytes = zip(&[
        ("first.bin", b"ok", CompressionMethod::Deflated),
        ("second.bin", b"bad", CompressionMethod::Deflated),
    ]);
    let compressed_size = offsets(&bytes, 1).compressed_size as u32;
    patch_sizes(&mut bytes, 1, compressed_size, 2);
    let mut archive = ProjectArchive::open(&bytes, limits(8, 2, 4, 1_000)).unwrap();
    assert_eq!(
        archive
            .read(&PackagePath::entry(b"first.bin").unwrap())
            .unwrap(),
        b"ok"
    );
    assert!(
        archive
            .read(&PackagePath::entry(b"second.bin").unwrap())
            .is_err()
    );
}

#[test]
fn project_archive_forces_crc_and_does_not_leak_payload() {
    let payload = b"secret payload bytes";
    let mut bytes = zip(&[("private.bin", payload, CompressionMethod::Stored)]);
    let at = offsets(&bytes, 0);
    bytes[at.data] ^= 1;
    let mut archive = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).unwrap();

    let error = archive
        .read(&PackagePath::entry(b"private.bin").unwrap())
        .unwrap_err()
        .to_string();

    assert!(error.contains("private.bin"));
    assert!(!error.contains("secret payload bytes"));
}

fn zip(entries: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for &(name, contents, method) in entries {
        writer
            .start_file(
                name,
                SimpleFileOptions::default().compression_method(method),
            )
            .unwrap();
        writer.write_all(contents).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn empty_zip(count: usize) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..count {
        writer
            .start_file(
                format!("{index}.empty"),
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn stream_zip(name: &str, contents: &[u8]) -> Vec<u8> {
    stream_zip_with_options(name, contents, SimpleFileOptions::default())
}

fn stream_zip_with_options(name: &str, contents: &[u8], options: SimpleFileOptions) -> Vec<u8> {
    let mut writer = ZipWriter::new_stream(Vec::new());
    writer.start_file(name, options).unwrap();
    writer.write_all(contents).unwrap();
    writer.finish().unwrap().into_inner()
}

fn zip_with_unicode_alias(physical: &str, alias: &str) -> Vec<u8> {
    let mut unicode_path = vec![1];
    unicode_path.extend_from_slice(&0u32.to_le_bytes());
    unicode_path.extend_from_slice(alias.as_bytes());
    let mut options = SimpleFileOptions::default().into_full_options();
    options.add_extra_data(0x7075, unicode_path, true).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer.start_file(physical, options).unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    let central = bytes
        .windows(4)
        .position(|window| window == b"PK\x01\x02")
        .unwrap();
    let name_len = u16::from_le_bytes(bytes[central + 28..central + 30].try_into().unwrap());
    let unicode_extra = central + 46 + usize::from(name_len);
    assert_eq!(&bytes[unicode_extra..unicode_extra + 2], b"up");
    write_u32(&mut bytes, unicode_extra + 5, crc32(physical.as_bytes()));
    bytes
}

fn limits(entries: usize, entry: u64, total: u64, ratio: u64) -> ArchiveLimits {
    ArchiveLimits {
        max_entries: entries,
        max_entry_size: entry,
        max_total_size: total,
        max_expansion_ratio: ratio,
    }
}

#[derive(Clone, Copy)]
struct Offsets {
    header: usize,
    data: usize,
    central: usize,
    compressed_size: u64,
}

fn offsets(bytes: &[u8], index: usize) -> Offsets {
    let mut archive = RawZipArchive::new(Cursor::new(bytes)).unwrap();
    let file = archive.by_index_raw(index).unwrap();
    Offsets {
        header: file.header_start() as usize,
        data: file.data_start().unwrap() as usize,
        central: file.central_header_start() as usize,
        compressed_size: file.compressed_size(),
    }
}

fn patch_sizes(bytes: &mut [u8], index: usize, compressed: u32, expanded: u32) {
    let at = offsets(bytes, index);
    write_u32(bytes, at.header + 18, compressed);
    write_u32(bytes, at.header + 22, expanded);
    write_u32(bytes, at.central + 20, compressed);
    write_u32(bytes, at.central + 24, expanded);
}

fn assert_local_conflict(mutate: impl FnOnce(&mut [u8], Offsets)) {
    let mut bytes = zip(&[("a", b"data", CompressionMethod::Stored)]);
    let at = offsets(&bytes, 0);
    mutate(&mut bytes, at);
    assert!(ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).is_err());
}

fn set_flag(bytes: &mut [u8], offset: usize, flag: u16) {
    let value = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) | flag;
    write_u16(bytes, offset, value);
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
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
