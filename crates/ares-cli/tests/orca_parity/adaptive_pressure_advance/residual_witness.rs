//! Runtime Native reachability for GCode.cpp:5777-5806,5900-5973,6720-6745.

use super::settings;
use crate::{CaseInputs, OrcaRunner, runner};
use serde_json::json;
use std::fs;

#[test]
fn adaptive_pa_sloped_loop_native_witness() {
    check("all", "1");
}

#[test]
fn adaptive_pa_non_sloped_loop_control() {
    check("none", "1");
}

#[test]
fn adaptive_pa_sloped_loop_disabled_control() {
    check("all", "0");
}

fn check(scarf: &str, adaptive: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("adaptive-pa-sloped-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let (machine, mut process, mut filaments) = settings();
    let process_values = [
        ("wall_generator", "classic"),
        ("wall_loops", "1"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("outer_wall_line_width", "0.4"),
        ("seam_slope_type", scarf),
        ("seam_slope_conditional", "0"),
        ("seam_slope_inner_walls", "0"),
        ("seam_slope_entire_loop", "0"),
        ("seam_slope_start_height", "0%"),
        ("seam_slope_min_length", "10"),
        ("seam_slope_steps", "4"),
        ("scarf_joint_speed", "100%"),
        ("seam_gap", "0"),
        ("seam_position", "nearest"),
        ("gcode_comments", "1"),
        ("spiral_mode", "0"),
        ("sparse_infill_density", "0%"),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("enable_arc_fitting", "0"),
        ("brim_type", "no_brim"),
        ("skirt_loops", "0"),
    ];
    let filament_values = [
        ("enable_pressure_advance", "1"),
        ("pressure_advance", "0.035"),
        ("adaptive_pressure_advance", adaptive),
        ("adaptive_pressure_advance_overhangs", "0"),
        ("adaptive_pressure_advance_bridges", "0"),
        (
            "adaptive_pressure_advance_model",
            "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000",
        ),
    ];
    for (key, value) in process_values {
        process.insert(key.into(), json!(value));
    }
    for filament in &mut filaments {
        for (key, value) in filament_values {
            filament.insert(key.into(), json!([value]));
        }
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("adaptive-pa/residual-r1/scarf-{scarf}/adaptive-{adaptive}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in process_values {
        runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    for (key, value) in filament_values {
        runner::application::verify_value(&exported.project, key, &json!([value])).unwrap();
    }
    fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    let pa: Vec<_> = native
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("; PA_CHANGE:") || line.starts_with("; APA "))
        .collect();
    fs::write(
        directory.join("native-pa-lines.json"),
        serde_json::to_vec_pretty(&json!({"zero_based_lines": pa})).unwrap(),
    )
    .unwrap();
    eprintln!(
        "Adaptive PA Native witness artifacts: {}",
        directory.display()
    );
    assert_eq!(native.contains("; PA_CHANGE:T0 "), adaptive == "1");
    assert!(!native.contains(";TYPE:Prime tower\n"));
    let layer = native.split(";LAYER_CHANGE\n").nth(2).unwrap();
    let markers: Vec<_> = layer
        .lines()
        .filter(|line| line.starts_with("; PA_CHANGE:"))
        .collect();
    let moves: Vec<_> = layer
        .lines()
        .filter(|line| {
            line.starts_with("G1 ")
                && line.contains('X')
                && line.contains('Y')
                && line.contains('E')
                && line.ends_with("; perimeter")
        })
        .collect();
    let rising_z: Vec<f64> = moves
        .iter()
        .filter_map(|line| {
            line.split_whitespace()
                .find_map(|word| word.strip_prefix('Z'))
                .map(|value| value.parse().unwrap())
        })
        .collect();
    if scarf == "all" {
        assert!(rising_z.len() > 1, "Native sloped starts absent");
        assert!(rising_z.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(moves.len() > rising_z.len(), "Native flat/ends absent");
        assert!(moves.last().unwrap().contains(" E0 "));
    } else {
        assert!(
            rising_z.is_empty(),
            "Native non-sloped control has slope moves"
        );
    }
    assert_eq!(markers.len(), usize::from(adaptive == "1"));
    if adaptive == "1" {
        assert!(layer.find(markers[0]).unwrap() < layer.find(moves[0]).unwrap());
    }
    fs::write(
        directory.join("reachability.json"),
        serde_json::to_vec_pretty(&json!({
            "layer": 1, "scarf": scarf, "adaptive": adaptive,
            "pa_markers": markers, "rising_z": rising_z, "moves": moves,
            "source": "ExtrusionEntity.cpp:426-525 starts/flat/ends; GCode.cpp:5951-5966,6720-6745",
            "scope": "uniform-width no-tower same-role slope; not heterogeneous-flow or role-change proof"
        }))
        .unwrap(),
    )
    .unwrap();
    let outcome = crate::compare_case(&sliced);
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({"status": outcome.status, "detail": outcome.detail}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
}
