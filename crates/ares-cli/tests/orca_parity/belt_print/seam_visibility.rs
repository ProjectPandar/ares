use serde_json::{Map, Value, json};
use std::io::{Cursor, Read, Write};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const TWO_HOLES: &[u8] =
    include_bytes!("../../../../../tests/parity/inner-only-brim/two-holes.3mf");

#[test]
fn aligned_seam_on_a_tilted_belt_matches_native() {
    super::compare_project("belt-aligned-seam", project(true, "aligned"));
}

#[test]
fn aligned_back_seam_on_a_tilted_belt_matches_native() {
    super::compare_project("belt-aligned-back-seam", project(true, "aligned_back"));
}

#[test]
fn nearest_seam_on_a_tilted_belt_matches_native() {
    super::compare_project("belt-nearest-seam", project(true, "nearest"));
}

#[test]
fn rear_seam_on_a_tilted_belt_matches_native() {
    super::compare_project("belt-rear-seam", project(true, "back"));
}

#[test]
fn random_seam_on_a_tilted_belt_matches_native() {
    super::compare_project("belt-random-seam", project(true, "random"));
}

#[test]
fn aligned_seam_on_a_flat_bed_matches_native() {
    super::compare_project("flat-aligned-seam", project(false, "aligned"));
}

fn project(belt: bool, seam: &str) -> Vec<u8> {
    let mut reader = ZipArchive::new(Cursor::new(TWO_HOLES)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..reader.len() {
        let mut member = reader.by_index(i).unwrap();
        let name = member.name().to_owned();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        let bytes = match name.as_str() {
            "Metadata/project_settings.config" => settings(&bytes, belt, seam),
            "3D/3dmodel.model" => {
                let model = String::from_utf8(bytes).unwrap();
                let original = "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"";
                assert_eq!(model.matches(original).count(), 1);
                model
                    .replacen(original, "transform=\"1 0 0 0 0.8 0 0 0 2 0 0 0\"", 1)
                    .into_bytes()
            }
            _ => bytes,
        };
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn settings(bytes: &[u8], belt: bool, seam: &str) -> Vec<u8> {
    let mut config: Map<String, Value> = serde_json::from_slice(bytes).unwrap();
    for (key, value) in [
        ("belt_printer", if belt { "1" } else { "0" }),
        ("belt_slice_rotation", "x"),
        ("belt_slice_rotation_angle", "1"),
        ("gcode_remap_x", "rev_x"),
        ("gcode_remap_y", "pos_z"),
        ("gcode_remap_z", "pos_y"),
        ("enable_arc_fitting", "disabled"),
        ("brim_type", "no_brim"),
        ("seam_position", seam),
        ("initial_layer_print_height", "0.3"),
        ("layer_height", "0.3"),
    ] {
        config.insert(key.to_owned(), json!(value));
    }
    serde_json::to_vec(&config).unwrap()
}
