//! Custom G-code motion-state invalidation: `GCode::placeholder_parser_process` drops the writer's
//! acceleration and jerk caches when the expanded G-code sets them itself (`GCode.cpp:842-876,
//! 5362-5366`, `GCodeWriter.hpp:165-166`, v2.5.0-beta `345efec64`)
//! (`docs/architecture/custom-gcode-motion-state.md`). A section oracle: each layer's acceleration
//! and jerk lines, placed among its role lines, must match Native's; the rest of the output is not
//! compared.

use super::rewrite_project;
use crate::runner::{OrcaRunner, repo_root};

const MK4: &str = "tests/parity/mk4-layer-change/mk4base.3mf";
const M201: &str = "\\nM201 X{interpolate_table(extruded_weight_total, (0,4000), (1400,2500), (10000,2500))} Y{interpolate_table(extruded_weight_total, (0,4000), (1400,2500), (10000,2500))}";
const BEFORE_LAYER_CHANGE: &str =
    "\"before_layer_change_gcode\": \";BEFORE_LAYER_CHANGE\\nG92 E0.0\\n;[layer_z]";
/// The tower is off: its final purge is a separate slice and would otherwise differ on the last layer.
const NO_TOWER: (&str, &str) = (
    "\"enable_prime_tower\": \"1\"",
    "\"enable_prime_tower\": \"0\"",
);
const LAYER_CHANGE: &str = "\"layer_change_gcode\": \";AFTER_LAYER_CHANGE\\n;[layer_z]";

/// Per `;LAYER_CHANGE`, the acceleration and jerk lines in order, with the `;TYPE:` lines between.
fn motion_state_lines(gcode: &[u8]) -> Vec<Vec<&str>> {
    let mut layers = Vec::new();
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layers.push(Vec::new());
        } else if let Some(layer) = layers.last_mut()
            && (line.starts_with(";TYPE:")
                || ["M201", "M202", "M204", "M205", "SET_VELOCITY_LIMIT"]
                    .iter()
                    .any(|command| line.split_whitespace().next() == Some(command)))
        {
            layer.push(line);
        }
    }
    assert!(layers.len() > 1, "the G-code has no layer changes");
    layers
}

fn assert_motion_state_matches(label: &str, edits: &[(&str, &str)]) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0-beta slicing oracle");
    let edits = [edits, &[NO_TOWER]].concat();
    let project = rewrite_project(&std::fs::read(repo_root().join(MK4)).unwrap(), &edits);
    let exported = runner
        .fixture_case(&format!("custom-gcode-motion-state/{label}"), project)
        .unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    let output = exported.output_dir.join("ares.cli.gcode");
    let cli = crate::runner::application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{label}: CLI {cli:?}");
    let ares = std::fs::read(output).unwrap();
    assert_eq!(
        motion_state_lines(&ares),
        motion_state_lines(&native.reference),
        "{label}: acceleration and jerk lines"
    );
}

#[test]
fn before_layer_change_acceleration_resets_the_writer_like_native() {
    assert_motion_state_matches("before-layer-change-m201", &[]);
}

#[test]
fn layer_change_acceleration_resets_the_writer_like_native() {
    let before = format!("{BEFORE_LAYER_CHANGE}{M201}");
    let after = format!("{LAYER_CHANGE}\\nM201 X4000 Y4000");
    assert_motion_state_matches(
        "layer-change-m201",
        &[(&before, BEFORE_LAYER_CHANGE), (LAYER_CHANGE, &after)],
    );
}

#[test]
fn layer_change_jerk_resets_the_writer_like_native() {
    let after = format!("{LAYER_CHANGE}\\nM205 X8 Y8");
    assert_motion_state_matches("layer-change-m205", &[(LAYER_CHANGE, &after)]);
}

#[test]
fn custom_gcode_without_motion_commands_keeps_the_writer_like_native() {
    let before = format!("{BEFORE_LAYER_CHANGE}{M201}");
    assert_motion_state_matches("no-motion-commands", &[(&before, BEFORE_LAYER_CHANGE)]);
}
