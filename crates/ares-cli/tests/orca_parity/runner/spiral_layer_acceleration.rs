//! Spiral layers set the travel acceleration on their zero-length change-layer travel
//! (`GCode.cpp:9224-9246`, `:10372-10418`) after the `M201` in `before_layer_change_gcode`
//! invalidated it (`GCode.cpp:5362-5366`) (`docs/architecture/change-layer-travel.md`).

use std::process::Command;

use crate::runner::{OrcaRunner, repo_root};

/// Per `;LAYER_CHANGE`, the `M204` lines in order.
fn acceleration_lines(gcode: &[u8]) -> Vec<Vec<&str>> {
    let mut layers = Vec::new();
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layers.push(Vec::new());
        } else if let Some(layer) = layers.last_mut()
            && line.split_whitespace().next() == Some("M204")
        {
            layer.push(line);
        }
    }
    assert!(layers.len() > 1, "the G-code has no layer changes");
    layers
}

/// A section oracle: the spiral layers' zero-length change-layer travel sets the travel
/// acceleration, which the `M201` in `before_layer_change_gcode` has just invalidated
/// (`GCode.cpp:5362-5366`). The rest of the output is not compared.
fn assert_spiral_acceleration_matches(label: &str, fixture: &str) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0-beta slicing oracle");
    let project = std::fs::read(repo_root().join(fixture)).unwrap();
    let exported = runner
        .fixture_case(&format!("spiral-layer-acceleration/{label}"), project)
        .unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    let output = exported.output_dir.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{label}: CLI {cli:?}");
    let ares = std::fs::read(output).unwrap();
    assert_eq!(
        acceleration_lines(&ares),
        acceleration_lines(&native.reference),
        "{label}: M204 lines per layer"
    );
}

#[test]
fn spiral_hole_layers_set_the_travel_acceleration_like_native() {
    assert_spiral_acceleration_matches(
        "spiral-hole",
        "tests/parity/spiral-hole-projected-split/spiral.3mf",
    );
}

#[test]
fn spiral_active_ranges_set_the_travel_acceleration_like_native() {
    assert_spiral_acceleration_matches(
        "spiral-active-ranges",
        "tests/parity/spiral-layer-ranges/active.3mf",
    );
}

#[test]
fn spiral_ignored_ranges_set_the_travel_acceleration_like_native() {
    assert_spiral_acceleration_matches(
        "spiral-ignored-ranges",
        "tests/parity/spiral-layer-ranges/ignored.3mf",
    );
}
