use std::fmt::Write as _;
use std::fs;
use std::io::{Cursor, Write as _};
use std::path::Path;

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{comparator, invoke, write_saved_input};
use crate::OrcaRunner;

const PREVIEWS: [(&str, &str); 4] = [
    ("thumbnail_file", "Metadata/plate_1.png"),
    ("thumbnail_no_light_file", "Metadata/plate_no_light_1.png"),
    ("top_file", "Metadata/top_1.png"),
    ("pick_file", "Metadata/pick_1.png"),
];
const SMALL: &str = "Metadata/plate_1_small.png";
const CUT: &str = "Metadata/cut_information.xml";
const INERT_CUT: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<objects>
 <object id="1">
  <cut_id id="0" check_sum="0" connectors_cnt="0"/>
 </object>
</objects>
"#;
// Lossless encoding of one synthetic pixel (25, 119, 231, 255).
const WEBP: &[u8] =
    b"RIFF\x1e\0\0\0WEBPVP8L\x11\0\0\0/\0\0\0\0\x07\xd0\xbbf\xf4\xbc\xff\x81\x88\xe8\x7f\0\0";

#[test]
fn saved_plate_previews_and_package_cover_match_native() {
    check(
        "Auxiliaries/.thumbnails/thumbnail_3mf.png",
        &patterned_png(2, 17),
    );
}

#[test]
fn saved_plate_previews_and_middle_cover_match_native() {
    check(
        "Auxiliaries/.thumbnails/thumbnail_middle.png",
        &patterned_png(2, 19),
    );
}

#[test]
fn saved_plate_previews_and_small_cover_match_native() {
    check(
        "Auxiliaries/.thumbnails/thumbnail_small.png",
        &patterned_png(2, 23),
    );
}

#[test]
fn saved_plate_previews_and_model_picture_match_native() {
    check("Auxiliaries/Model Pictures/étude.webp", WEBP);
}

#[test]
fn saved_plate_previews_and_profile_picture_match_native() {
    check("Auxiliaries/Profile Pictures/swatch.webp", WEBP);
}

fn check(auxiliary: &str, auxiliary_bytes: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(source.join("out")).unwrap();
    write_saved_input(&runner, &source);
    let original = fs::read(source.join("saved.3mf")).unwrap();
    let primary = patterned_png(256, 0);
    let secondary = [
        patterned_png(4, 21),
        patterned_png(4, 22),
        patterned_png(4, 23),
    ];
    let project = with_saved_assets(&original, &primary, &secondary, auxiliary, auxiliary_bytes);

    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    let argv = ["--slice", "1", "--export-3mf", "export.3mf", "saved.3mf"];
    let export = |bin: &Path, cwd: &Path| {
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join("saved.3mf"), &project).unwrap();
        assert_eq!(
            invoke(bin, cwd, &argv),
            Some(0),
            "{} failed in {}",
            bin.display(),
            cwd.display()
        );
        assert!(!cwd.join("plate_1.gcode").exists());
        assert_eq!(fs::read_dir(cwd.join("out")).unwrap().count(), 0);
        comparator::read_members(&cwd.join("export.3mf"))
    };
    let native = export(runner.bin(), &native_cwd);
    let member = |name| native.iter().find(|entry| entry.name == name).unwrap();
    assert_ne!(member(PREVIEWS[0].1).bytes, primary);
    assert_eq!(&member(SMALL).bytes[16..24], &[0, 0, 0, 128, 0, 0, 0, 128]);
    for ((_, name), expected) in PREVIEWS[1..].iter().zip(&secondary) {
        assert_eq!(&member(name).bytes, expected, "{name}");
    }
    assert_eq!(member(auxiliary).bytes, auxiliary_bytes);
    assert!(native.iter().all(|entry| entry.name != CUT));

    let ares = export(Path::new(env!("CARGO_BIN_EXE_ares")), &ares_cwd);
    comparator::compare_members(native, ares);
}

fn with_saved_assets(
    input: &[u8],
    primary: &[u8],
    secondary: &[Vec<u8>; 3],
    auxiliary: &str,
    bytes: &[u8],
) -> Vec<u8> {
    let input = rewrite_project(input, |name, xml| {
        if name != "Metadata/model_settings.config" {
            return xml.to_owned();
        }
        let mut fields = String::new();
        for (key, path) in PREVIEWS {
            writeln!(fields, "    <metadata key=\"{key}\" value=\"{path}\"/>").unwrap();
        }
        assert_eq!(xml.matches("    <model_instance>").count(), 1);
        xml.replace(
            "    <model_instance>",
            &format!("{fields}    <model_instance>"),
        )
    });
    let mut archive = ZipWriter::new_append(Cursor::new(input)).unwrap();
    for (path, payload) in PREVIEWS
        .map(|(_, path)| path)
        .into_iter()
        .zip(std::iter::once(primary).chain(secondary.iter().map(Vec::as_slice)))
        .chain([(CUT, INERT_CUT), (auxiliary, bytes)])
    {
        archive
            .start_file(path, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(payload).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

fn patterned_png(edge: u16, phase: u8) -> Vec<u8> {
    let mut pixels = Vec::new();
    for y in 0..edge {
        pixels.push(0); // no row filter
        for x in 0..edge {
            pixels.extend_from_slice(&[
                (x * 5 + y * 3) as u8,
                (y * 7 + x % 2) as u8,
                ((x ^ (y * 11)) as u8).wrapping_add(phase),
                255 - (x * 3 + y * 5 + x % 2) as u8,
            ]);
        }
    }
    // Stored DEFLATE blocks deliberately differ from the exporter's PNG encoding.
    let mut zlib = vec![0x78, 0x01];
    let blocks = pixels.chunks(u16::MAX as usize);
    let count = blocks.len();
    for (index, block) in blocks.enumerate() {
        let size = block.len() as u16;
        zlib.push(u8::from(index + 1 == count));
        zlib.extend_from_slice(&size.to_le_bytes());
        zlib.extend_from_slice(&(!size).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    let (a, b) = pixels.iter().fold((1_u32, 0_u32), |(a, b), &byte| {
        let a = (a + u32::from(byte)) % 65521;
        (a, (b + a) % 65521)
    });
    zlib.extend_from_slice(&(b << 16 | a).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let dimension = u32::from(edge).to_be_bytes();
    let header = [dimension.as_slice(), &dimension, &[8, 6, 0, 0, 0]].concat();
    for (tag, data) in [
        (b"IHDR", header.as_slice()),
        (b"IDAT", &zlib),
        (b"IEND", &[][..]),
    ] {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let start = png.len();
        png.extend_from_slice(tag);
        png.extend_from_slice(data);
        let crc = png[start..].iter().fold(u32::MAX, |crc, &byte| {
            (0..8).fold(crc ^ u32::from(byte), |crc, _| {
                (crc >> 1) ^ (0xedb8_8320 * (crc & 1))
            })
        });
        png.extend_from_slice(&(!crc).to_be_bytes());
    }
    png
}
