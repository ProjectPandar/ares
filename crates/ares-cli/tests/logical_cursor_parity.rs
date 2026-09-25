//! `GCode::extrude_loop` writer-only motion followed by `extrude_infill` chaining.
use std::{fs, path::Path, process::Command};

#[test]
fn extrude_loop_preserves_complete_ordered_sparse_runs_and_inward_move() {
    let (actual, expected) = slice_fixture("anchor");
    for z in ["2.58", "2.72", "8.04"] {
        let actual = layer(&actual, z);
        let expected = layer(&expected, z);
        assert!(expected.contains("G1 X164.851 Y179.88\n"));
        assert!(expected.contains(";TYPE:Sparse infill\n"));
        // Compare the entire layer, including the inward move, retraction,
        // approach, both sparse paths, intervening travel, feeds and M73.
        assert_eq!(actual, expected, "complete layer at Z{z}");
    }
}

#[test]
fn two_wall_inward_move_preserves_orca_layer_sections() {
    let (actual, expected) = slice_fixture("two-walls");
    for z in ["0.34", "0.76", "1.18"] {
        assert_eq!(
            layer(&actual, z),
            layer(&expected, z),
            "complete layer at Z{z}"
        );
    }
}

fn slice_fixture(name: &str) -> (String, String) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/avoid_crossing_travel");
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("actual.gcode");
    let binary =
        std::env::var_os("ARES_TRAVEL_BIN").unwrap_or_else(|| env!("CARGO_BIN_EXE_ares").into());
    let result = Command::new(binary)
        .arg("slice")
        .arg("-o")
        .arg(&output)
        .arg(fixtures.join(format!("{name}.3mf")))
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    (
        fs::read_to_string(output).unwrap(),
        fs::read_to_string(fixtures.join(format!("{name}.orca.gcode"))).unwrap(),
    )
}

fn layer<'a>(gcode: &'a str, z: &str) -> &'a str {
    let marker = format!(";Z:{z}\n");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    &rest[..rest.find(";LAYER_CHANGE\n").unwrap()]
}
