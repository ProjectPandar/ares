//! Approved APA-R2 admission for classic sloped-loop PA context.

use super::settings;
use crate::{CaseInputs, OrcaRunner, runner};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

const MESH_SHA256: &str = "a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031";
const MODEL: &str = "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000";

#[test]
fn adaptive_pa_sloped_loop_n0() {
    check("none", "0", "0");
}

#[test]
fn adaptive_pa_sloped_loop_n1() {
    check("none", "1", "0");
}

#[test]
fn adaptive_pa_sloped_loop_n2() {
    check("none", "1", "1");
}

fn check(scarf: &str, adaptive: &str, overhang: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("apa-sloped-n{adaptive}o{overhang}-{scarf}-"))
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
                    "adaptive-pa/r2-m/sloped/scarf-{scarf}/adaptive-{adaptive}/overhang-{overhang}"
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
    let admission = json!({
        "scarf": scarf,
        "adaptive": adaptive,
        "overhang": overhang,
        "pa_change": native
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with("; PA_CHANGE:"))
            .collect::<Vec<_>>(),
        "rising_z": native
            .lines()
            .filter(|line| line.starts_with("G1 ") && line.contains(" Z"))
            .take(80)
            .collect::<Vec<_>>(),
        "scope": "admission artifact; full source reachability audit is external"
    });
    fs::write(
        directory.join("admission.json"),
        serde_json::to_vec_pretty(&admission).unwrap(),
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
