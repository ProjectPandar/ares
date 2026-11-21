//! Native parity for printed Grid and its bridge anchors over an odd raft.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value};

#[test]
fn odd_raft_preserves_native_trapezoidal_grid_and_bridge_motion() {
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
        ("sparse_infill_pattern", "grid"),
        ("fill_multiline", "2"),
        ("infill_direction", "90"),
        ("infill_combination", "0"),
        ("raft_layers", "3"),
        ("support_type", "normal(auto)"),
        ("enable_arc_fitting", "0"),
    ];
    for (key, value) in options {
        process.insert(key.into(), Value::from(value));
    }
    let exported = orca
        .export_case(
            &CaseInputs {
                label: "trapezoidal-grid-over-odd-raft",
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
    let native = std::str::from_utf8(&case.reference).unwrap();
    for role in ["Support", "Sparse infill", "Internal Bridge"] {
        assert!(
            native
                .lines()
                .any(|line| line.strip_prefix("; FEATURE: ") == Some(role)),
            "missing Native {role}"
        );
    }
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
