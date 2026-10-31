//! `infill_anchor < 0` upstream admission (docs/architecture/infill-anchor-negative.md).
//!
//! `infill_anchor` is a boundless `coFloatOrPercent`
//! (`PrintConfig.cpp:3137-3160`); upstream projects
//! `anchor_length = min(anchor, anchor_max)` with no sign check
//! (`Fill.cpp:984-988`), and the `anchor_length >= 0.` asserts
//! (`FillBase.cpp:1584-1585,2251-2252`) are debug-only — the release oracle
//! slices negative anchors and defines their semantics. Ares' admission guard
//! rejects the whole domain (`UnsupportedProjectFeature("infill_anchor")`).

use crate::{
    CaseInputs, OrcaRunner, VendorProfiles, normalize_filament_defaults,
    normalize_process_defaults, runner, select_printer, smoke_overrides,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

const MESH_SHA256: &str = "a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031";

#[test]
fn infill_anchor_negative_one_connect() {
    check("-1", "1000");
}

#[test]
fn infill_anchor_negative_one_dont_connect() {
    check("-1", "0");
}

#[test]
fn infill_anchor_negative_percent_connect() {
    check("-50%", "1000");
}

fn check(anchor: &str, anchor_max: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("infill-anchor-neg-{anchor}-max{anchor_max}-"))
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
        ("sparse_infill_pattern", Value::from("rectilinear")),
        ("sparse_infill_density", Value::from("20%")),
        ("infill_anchor", Value::from(anchor)),
        ("infill_anchor_max", Value::from(anchor_max)),
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
                label: &format!("infill-anchor/negative/{anchor}/max-{anchor_max}"),
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
    let infill_lines = native
        .lines()
        .filter(|line| line.contains("infill") && line.starts_with('G'))
        .count();
    fs::write(
        directory.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "infill_anchor": anchor,
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
