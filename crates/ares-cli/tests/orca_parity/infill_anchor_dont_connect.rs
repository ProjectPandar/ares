//! `infill_anchor_max < 0.05` upstream `dont_connect` admission
//! (docs/architecture/infill-anchor-dont-connect.md).
//!
//! Upstream `FillParams::dont_connect()` (`FillBase.hpp:52`) selects the
//! `chain_polylines` + append path (`FillRectilinear.cpp:3366-3373`,
//! `FillBase.cpp:1820-1828`) for `anchor_length_max < 0.05` — including the
//! absolute `0` that `PrintConfig.cpp:3137-3185` exposes as a documented enum
//! value. Ares' admission guard at
//! `bridge_over_infill/transaction/anchor_projection.rs` wrongly rejects these
//! inputs (`UnsupportedProjectFeature("infill_anchor_max")`) even though the
//! ported dont_connect plumbing already exists.

use crate::{
    CaseInputs, OrcaRunner, VendorProfiles, normalize_filament_defaults,
    normalize_process_defaults, runner, select_printer, smoke_overrides,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

const MESH_SHA256: &str = "a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031";

#[test]
fn infill_anchor_max_zero_rectilinear() {
    check("rectilinear", "0");
}

#[test]
fn infill_anchor_max_zero_zigzag() {
    check("zigzag", "0");
}

#[test]
fn infill_anchor_max_below_threshold_rectilinear() {
    check("rectilinear", "0.02");
}

fn check(pattern: &str, anchor_max: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("infill-anchor-{pattern}-{anchor_max}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection = select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    normalize_filament_defaults(&mut filaments);
    let process_values = [
        ("sparse_infill_pattern", Value::from(pattern)),
        ("sparse_infill_density", Value::from("20%")),
        ("infill_anchor_max", Value::from(anchor_max)),
        ("infill_anchor", Value::from("0")),
        ("fill_multiline", Value::from("1")),
        ("enable_prime_tower", Value::from("0")),
        ("gcode_comments", Value::from("1")),
    ];
    for (key, value) in &process_values {
        process.insert((*key).into(), value.clone());
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
                label: &format!("infill-anchor/dont-connect/{pattern}/max-{anchor_max}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &mesh,
        )
        .unwrap();
    for (key, value) in &process_values {
        runner::application::verify_value(&exported.project, key, value).unwrap();
    }
    fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    // Witness: the upstream binary produced sparse-infill output for a value
    // domain the Ares admission guard rejects outright.
    let infill_lines = native
        .lines()
        .filter(|line| line.contains("infill") && line.starts_with('G'))
        .count();
    fs::write(
        directory.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "pattern": pattern,
            "infill_anchor_max": anchor_max,
            "native_sparse_infill_gcode_lines": infill_lines,
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
