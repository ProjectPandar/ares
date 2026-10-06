//! Prewritten E2E controls for the source bed-region subtraction.
use std::io::{Cursor, Read, Write};

use serde_json::{Value, json};

use crate::brim_bed_region::compare_native;

const OUTER: &[u8] =
    include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.4-outer-only.3mf");

#[test]
fn brim_bed_control_ordinary_single_object_without_exclusion_matches_native() {
    let project = overlay(OUTER, &[("bed_exclude_area", None)]);
    compare_native("brim-ordinary-single-no-exclusion", &project, true);
}

#[test]
fn brim_bed_control_no_exclusion_nozzle_variants_match_native() {
    for (label, input) in [
        (
            "brim-no-exclusion-0.2",
            include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.2.3mf")
                .as_slice(),
        ),
        (
            "brim-no-exclusion-0.6",
            include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.6.3mf")
                .as_slice(),
        ),
        (
            "brim-no-exclusion-0.8",
            include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.8.3mf")
                .as_slice(),
        ),
    ] {
        let project = overlay(input, &[("bed_exclude_area", Some(json!([])))]);
        compare_native(label, &project, true);
    }
}

#[test]
fn brim_bed_control_original_auto_brim_is_inactive() {
    compare_native(
        "brim-original-auto-0.4-inactive",
        include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.4.3mf"),
        false,
    );
}

#[test]
fn brim_bed_control_explicit_tool_area_bypasses_model_bed_fallback() {
    let project = overlay(
        OUTER,
        &[(
            "extruder_printable_area",
            Some(json!(["0x0,255x0,255x255,0x255"])),
        )],
    );
    compare_native("brim-explicit-whole-bed-tool-area", &project, true);
}

#[test]
fn brim_bed_control_trailing_exclusion_pair_is_not_a_polygon() {
    let project = overlay(
        OUTER,
        &[(
            "bed_exclude_area",
            Some(json!([
                "0x0", "255x0", "255x255", "0x255", "0x0", "2x2", "2x253", "253x253", "150x150",
                "151x151",
            ])),
        )],
    );
    compare_native("brim-exclusion-trailing-pair", &project, true);
}

#[test]
fn brim_bed_control_clipped_footprint_reaches_start_template() {
    let settings = crate::runner::application::settings(OUTER).unwrap();
    let start = settings["machine_start_gcode"].as_str().unwrap();
    let project = overlay(
        OUTER,
        &[(
            "machine_start_gcode",
            Some(json!(format!(
                "{start}\n; clipped brim bounds {{first_layer_print_min[0]}} {{first_layer_print_min[1]}} {{first_layer_print_max[0]}} {{first_layer_print_max[1]}}\n"
            ))),
        )],
    );
    compare_native("brim-clipped-footprint-template", &project, true);
}

#[test]
fn brim_bed_control_empty_brim_does_not_reject_the_object() {
    let project = overlay(
        OUTER,
        &[(
            "extruder_printable_area",
            Some(json!(["105x105,110x105,110x110,105x110"])),
        )],
    );
    compare_native("brim-empty-clipped-area", &project, false);
}

fn overlay(input: &[u8], changes: &[(&str, Option<Value>)]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings = serde_json::from_slice::<Value>(&bytes).unwrap();
            let mut original = settings.clone();
            for (key, value) in changes {
                original.as_object_mut().unwrap().remove(*key);
                match value {
                    Some(value) => {
                        settings
                            .as_object_mut()
                            .unwrap()
                            .insert((*key).to_owned(), value.clone());
                    }
                    None => {
                        settings.as_object_mut().unwrap().remove(*key);
                    }
                }
            }
            let mut unchanged = settings.clone();
            for (key, _) in changes {
                unchanged.as_object_mut().unwrap().remove(*key);
            }
            assert_eq!(original, unchanged, "only requested config fields change");
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
