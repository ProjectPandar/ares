//! Public zero/near-zero sparse density under an internal solid ceiling.
//! `PrintObject.cpp:2521` retains unsupported internal areas at zero density;
//! `Fill.cpp:901-902` emits no sparse fill. Whole-output Native is authoritative.

use crate::{
    CaseInputs, OrcaRunner, VendorProfiles, normalize_filament_defaults,
    normalize_process_defaults, runner, select_printer, smoke_overrides,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn zero_density_internal_ceiling_matches_complete_native_output() {
    check("0%", false);
}

#[test]
fn near_zero_density_internal_ceiling_matches_complete_native_output() {
    check("0.0001%", false);
}

#[test]
fn positive_density_internal_ceiling_matches_complete_native_output() {
    check("20%", true);
}

fn check(density: &str, sparse_expected: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    let requested = [
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_density", density),
        ("layer_height", "0.2"),
        ("top_shell_layers", "5"),
        ("bottom_shell_layers", "3"),
        ("wall_loops", "2"),
        ("minimum_sparse_infill_area", "0"),
        ("dont_filter_internal_bridges", "disabled"),
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("exclude_object", "0"),
        ("gcode_comments", "1"),
    ];
    for (key, value) in requested {
        process.insert(key.into(), Value::from(value));
    }
    let mesh = runner::repo_root().join("tests/parity/cube10.stl");
    assert_eq!(
        hex(&fs::read(&mesh).unwrap()),
        "d66a155ffff024e9d09130d5394be47aaef509b31dbd4febddf32f9b88145968"
    );
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("bridge-zero-density/ceiling/{density}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &mesh,
        )
        .unwrap();
    for (key, value) in requested {
        runner::application::verify_value(&exported.project, key, &Value::from(value)).unwrap();
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let outcome = crate::compare_case(&sliced);
    let directory = outcome.artifacts.as_ref().unwrap();
    let (bridge_moves, sparse_moves) = moves(&sliced.reference);
    crate::artifacts::write(
        &directory.join("ceiling-witness.json"),
        &serde_json::to_vec_pretty(&json!({
            "requested_density": density,
            "expected_normalized_density": if sparse_expected {20.0} else {0.0},
            "native_internal_bridge_extrusion_moves": bridge_moves,
            "native_sparse_extrusion_moves": sparse_moves,
            "input_sha256": hex(&sliced.project),
            "native_sha256": hex(&sliced.reference),
            "status": outcome.status,
            "detail": outcome.detail,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        bridge_moves > 0,
        "Native internal ceiling witness: {directory:?}"
    );
    assert_eq!(
        sparse_moves > 0,
        sparse_expected,
        "Native sparse witness: {directory:?}"
    );
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
    let actual = fs::read(directory.join("ares.gcode")).unwrap();
    let cli = crate::runner::application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(directory.join("cli.gcode"))
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    crate::artifacts::write(&directory.join("cli.stdout"), &cli.stdout).unwrap();
    crate::artifacts::write(&directory.join("cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI slice: {directory:?}");
    let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
    assert_eq!(actual, cli_bytes, "API/CLI raw output: {directory:?}");
    crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes).unwrap();
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn moves(bytes: &[u8]) -> (usize, usize) {
    let mut role = "";
    let mut bridge = 0;
    let mut sparse = 0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && line.split_whitespace().any(|word| word.starts_with('E'))
            && line
                .split_whitespace()
                .any(|word| word.starts_with('X') || word.starts_with('Y'))
        {
            match role {
                "Internal Bridge" => bridge += 1,
                "Sparse infill" => sparse += 1,
                _ => {}
            }
        }
    }
    (bridge, sparse)
}
