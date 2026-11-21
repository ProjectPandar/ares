//! Native bridge output when a raft offsets the printed sparse infill layer ID.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value};
use std::fs;

#[test]
fn native_sparse_anchor_object_layer_triangles() {
    assert_native_bridge("triangles", "2", "1");
}

#[test]
fn native_sparse_anchor_object_layer_tri_hexagon() {
    assert_native_bridge("tri-hexagon", "2", "1");
}

fn assert_native_bridge(pattern: &str, multiline: &str, raft: &str) {
    let Some(orca) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner::repo_root();
    let profiles =
        VendorProfiles::load(&root.join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let machine = profiles.machine("Bambu Lab X1 Carbon 0.4 nozzle").unwrap();
    let mut process = profiles.process("0.20mm Standard @BBL X1C").unwrap();
    let mut filament = profiles.filament("Bambu PLA Basic @BBL X1C").unwrap();
    filament
        .entry("filament_is_support")
        .or_insert_with(|| serde_json::json!(["0"]));
    let options = [
        ("sparse_infill_pattern", pattern),
        ("fill_multiline", multiline),
        ("infill_direction", "90"),
        ("infill_combination", "0"),
        ("raft_layers", raft),
        ("support_type", "normal(auto)"),
        ("enable_arc_fitting", "0"),
    ];
    for (key, value) in options {
        process.insert(key.into(), Value::from(value));
    }
    let exported = orca
        .export_case(
            &CaseInputs {
                label: &format!("{pattern}-bridge-over-raft-{raft}"),
                machine: &machine,
                process: &process,
                filaments: &[filament],
            },
            &Map::new(),
            &root.join("tests/parity/grid-bridge-anchor-direction/plate.stl"),
        )
        .unwrap();
    for (key, value) in options {
        runner::application::verify_value(&exported.project, key, &Value::from(value)).unwrap();
    }
    let case = orca.slice_case(&exported).unwrap();
    let outcome = crate::compare_case(&case);
    let artifacts = outcome.artifacts.unwrap();
    let native = std::str::from_utf8(&case.reference).unwrap();
    let actual = fs::read_to_string(artifacts.join("ares.gcode")).unwrap();
    for role in ["Support interface", "Sparse infill", "Internal Bridge"] {
        let expected = feature_extrusions(native, role);
        assert!(!expected.is_empty(), "missing Native {role}: {artifacts:?}");
        let observed = feature_extrusions(&actual, role);
        if let Some((index, (left, right))) = expected
            .iter()
            .zip(&observed)
            .enumerate()
            .find(|(_, (a, b))| a != b)
        {
            panic!("{role} move {index}: Native {left:?}, Ares {right:?}; {artifacts:?}");
        }
        assert_eq!(observed.len(), expected.len(), "{role}: {artifacts:?}");
    }
}

fn feature_extrusions<'a>(gcode: &'a str, role: &str) -> Vec<(&'a str, &'a str)> {
    let mut height = "";
    let mut active = false;
    let mut result = Vec::new();
    for line in gcode.lines() {
        if let Some(z) = line.strip_prefix("; Z_HEIGHT: ") {
            height = z;
        }
        if let Some(feature) = line.strip_prefix("; FEATURE: ") {
            active = feature == role;
        }
        if active && let Some(command) = line.strip_prefix("G1 ") {
            let motion = command.split(';').next().unwrap().trim_end();
            let mut words = motion.split_ascii_whitespace();
            if words.clone().any(|word| word.starts_with(['X', 'Y']))
                && words.any(|word| {
                    word.strip_prefix('E')
                        .is_some_and(|e| e.parse::<f64>().unwrap() > 0.0)
                })
            {
                result.push((height, motion));
            }
        }
    }
    result
}
