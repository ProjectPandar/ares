//! Scarf "all" contexts S0/S1/S2 of the sloped-loop admission matrix
//! (docs/architecture/adaptive-pa-r2-m-sloped-loop-spec.md): per-point ramp
//! Z/E interpolation (`GCode.cpp:7114-7121,7149`) plus the inner-perimeter
//! slope front-clip boundary (`ExtrusionEntity.cpp:527-563`).

use super::settings;
use crate::{CaseInputs, OrcaRunner, runner};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

const MESH_SHA256: &str = "a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031";
const MODEL: &str = "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000";

#[test]
fn adaptive_pa_scarf_ramp_s0() {
    check("all", "0", "0", "1", "0");
}

#[test]
fn adaptive_pa_scarf_ramp_s1() {
    check("all", "1", "0", "1", "0");
}

#[test]
fn adaptive_pa_scarf_ramp_s2() {
    check("all", "1", "1", "1", "0");
}

/// Independent defect: upstream `ExtrusionLoopSloped::clip_front`
/// (`ExtrusionEntity.cpp:547-563`) only front-clips `starts` while
/// `distance = seam_gap * 2 > 0`; Ares clips 40% of slope length for
/// `erPerimeter` even at `seam_gap = 0`. This fixture puts the scarf on an
/// inner `erPerimeter` loop (`wall_loops = 2`, `seam_slope_inner_walls = 1`,
/// `seam_gap = 0`) where upstream emits the full unclipped slope head.
#[test]
fn adaptive_pa_scarf_ramp_inner_perimeter() {
    check("all", "0", "0", "2", "1");
}

fn check(scarf: &str, adaptive: &str, overhang: &str, wall_loops: &str, inner_walls: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("apa-scarf-ramp-a{adaptive}o{overhang}-{scarf}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    let (machine, mut process, mut filaments) = settings();
    let process_values = [
        ("wall_generator", "classic"),
        ("wall_loops", wall_loops),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("outer_wall_line_width", "0.4"),
        ("seam_slope_type", scarf),
        ("seam_slope_conditional", "0"),
        ("seam_slope_inner_walls", inner_walls),
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
        ("adaptive_pressure_advance_overhangs", overhang),
        ("adaptive_pressure_advance_bridges", "0"),
        ("adaptive_pressure_advance_model", MODEL),
    ];
    for (key, value) in process_values {
        process.insert(key.into(), json!(value));
    }
    for filament in &mut filaments {
        for (key, value) in filament_values {
            filament.insert(key.into(), json!([value]));
        }
    }
    let mesh = runner::repo_root().join("tests/parity/overhang.stl");
    let mesh_bytes = fs::read(&mesh).unwrap();
    assert_eq!(
        Sha256::digest(&mesh_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        MESH_SHA256
    );
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!(
                    "adaptive-pa/r2-m/scarf-ramp/{scarf}/adaptive-{adaptive}/overhang-{overhang}"
                ),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &mesh,
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
    assert!(!native.contains(";TYPE:Prime tower\n"));
    let outcome = crate::compare_case(&sliced);
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({"status": outcome.status, "detail": outcome.detail}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
}
