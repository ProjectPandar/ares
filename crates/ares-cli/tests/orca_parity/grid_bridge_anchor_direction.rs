//! Directed extrusion over the unsupported span of the synthetic Grid plate.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value};
use std::fs;

#[test]
fn grid_anchors_preserve_native_internal_bridge_traversal() {
    let Some(orca) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let stock = slice(&orca, "45", "0");
    let stock_outcome = crate::compare_case(&stock);
    assert_eq!(stock_outcome.status, "PASS", "{}", stock_outcome.detail);
    let native_stock = feature_moves(&stock.reference, "Internal Bridge", 5.0);
    assert!(!native_stock.is_empty(), "missing Native stock bridge");

    let rotated = slice(&orca, "90", "0");
    let native_rotated = assert_bridge_traversal(&rotated, 5.0);
    assert_ne!(
        native_rotated, native_stock,
        "Native direction did not activate"
    );
}

#[test]
fn odd_raft_preserves_native_grid_bridge_traversal() {
    let Some(orca) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let raft = slice(&orca, "90", "3");
    assert!(
        !feature_moves(&raft.reference, "Support", 0.2).is_empty(),
        "missing Native first-layer raft extrusion"
    );
    // Three raft layers flip the absolute ID's parity, but not Fill::layer_id.
    assert_bridge_traversal(&raft, 5.9);
}

fn slice(orca: &OrcaRunner, direction: &str, raft_layers: &str) -> runner::ParityCase {
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
    let mut requested = vec![
        ("infill_direction", direction),
        ("raft_layers", raft_layers),
    ];
    if raft_layers != "0" {
        // Isolate Grid anchoring from tree raft generation and interface arc fitting.
        requested.extend([
            ("support_type", "normal(auto)"),
            ("enable_arc_fitting", "0"),
        ]);
    }
    for &(key, value) in &requested {
        process.insert(key.into(), Value::from(value));
    }
    let exported = orca
        .export_case(
            &CaseInputs {
                label: &format!("grid-bridge-anchor-direction/{direction}/raft-{raft_layers}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &mesh,
        )
        .unwrap();
    for (key, value) in requested
        .into_iter()
        .chain([("sparse_infill_pattern", "grid")])
    {
        runner::application::verify_value(&exported.project, key, &Value::from(value)).unwrap();
    }
    orca.slice_case(&exported).unwrap()
}

fn assert_bridge_traversal(sliced: &runner::ParityCase, height: f64) -> Vec<(Point, Point)> {
    let outcome = crate::compare_case(sliced);
    let artifacts = outcome.artifacts.unwrap();
    let native = feature_moves(&sliced.reference, "Internal Bridge", height);
    assert!(!native.is_empty(), "missing Native bridge: {artifacts:?}");
    let actual = feature_moves(
        &fs::read(artifacts.join("ares.gcode")).unwrap(),
        "Internal Bridge",
        height,
    );
    assert_eq!(actual, native, "directed internal bridge: {artifacts:?}");
    assert_eq!(outcome.status, "PASS", "{}; {artifacts:?}", outcome.detail);
    native
}

type Point = (f64, f64);

fn feature_moves(bytes: &[u8], role: &str, height: f64) -> Vec<(Point, Point)> {
    let mut z = 0.0;
    let mut selected = false;
    let mut current = (0.0, 0.0);
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(height) = line.strip_prefix("; Z_HEIGHT: ") {
            z = height.parse().unwrap();
        }
        if let Some(feature) = line.strip_prefix("; FEATURE: ") {
            selected = feature == role;
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let Some("G0" | "G1" | "G2" | "G3") = words.next() else {
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
        if z == height && selected && extrudes && next != current {
            result.push((current, next));
        }
        current = next;
    }
    result
}
