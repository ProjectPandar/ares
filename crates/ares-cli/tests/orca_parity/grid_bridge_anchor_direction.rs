//! Directed extrusion over the unsupported span of the synthetic Grid plate.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value};
use std::fs;

#[test]
fn grid_anchors_preserve_native_internal_bridge_traversal() {
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
    let filaments = [filament];
    let mesh = root.join("tests/parity/grid-bridge-anchor-direction/plate.stl");
    let mut native_stock = Vec::new();
    for direction in ["45", "90"] {
        process.insert("infill_direction".into(), Value::from(direction));
        let exported = orca
            .export_case(
                &CaseInputs {
                    label: &format!("grid-bridge-anchor-direction/{direction}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &Map::new(),
                &mesh,
            )
            .unwrap();
        for (key, value) in [
            ("sparse_infill_pattern", "grid"),
            ("infill_direction", direction),
        ] {
            runner::application::verify_value(&exported.project, key, &Value::from(value)).unwrap();
        }
        let sliced = orca.slice_case(&exported).unwrap();
        let outcome = crate::compare_case(&sliced);
        let artifacts = outcome.artifacts.unwrap();
        let native = bridge_moves(&sliced.reference);
        assert!(!native.is_empty(), "missing Native bridge: {artifacts:?}");
        if direction == "45" {
            assert_eq!(outcome.status, "PASS", "{}; {artifacts:?}", outcome.detail);
            native_stock = native;
        } else {
            assert_ne!(native, native_stock, "Native direction did not activate");
            let actual = bridge_moves(&fs::read(artifacts.join("ares.gcode")).unwrap());
            assert_eq!(actual, native, "directed Z5 internal bridge: {artifacts:?}");
        }
    }
}

type Point = (f64, f64);

fn bridge_moves(bytes: &[u8]) -> Vec<(Point, Point)> {
    let mut z = 0.0;
    let mut internal_bridge = false;
    let mut current = (0.0, 0.0);
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(height) = line.strip_prefix("; Z_HEIGHT: ") {
            z = height.parse().unwrap();
        }
        if let Some(feature) = line.strip_prefix("; FEATURE: ") {
            internal_bridge = feature == "Internal Bridge";
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let Some(command @ ("G0" | "G1" | "G2" | "G3")) = words.next() else {
            continue;
        };
        let mut next = current;
        let mut extrudes = false;
        for word in words {
            if let Some(x) = word.strip_prefix('X') {
                next.0 = x.parse().unwrap();
            } else if let Some(y) = word.strip_prefix('Y') {
                next.1 = y.parse().unwrap();
            } else if let Some(e) = word.strip_prefix('E') {
                extrudes = e.parse::<f64>().unwrap() > 0.0;
            }
        }
        if z == 5.0 && internal_bridge && extrudes && next != current {
            assert_eq!(command, "G1");
            result.push((current, next));
        }
        current = next;
    }
    result
}
