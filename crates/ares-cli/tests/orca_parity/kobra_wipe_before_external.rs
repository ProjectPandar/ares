//! Anycubic Kobra wipe-before-external witness (tests-only RED for the
//! machine-matrix2 residual).
//!
//! Stock Anycubic Kobra 0.4 nozzle defaults + the hole-to-polyhole block
//! mesh. On layer Z:8 the inner→outer transition travel endpoint
//! prints `G1 X106.304 Y105.892 F10800` in Native and `Y105.891` in
//! Ares — the only hunk in 376402 bytes, stable across Native reruns
//! and identical in official and seeded modes.
//!
//! Root cause (probe-native pinpoint, `WIPE_DUMP` instrumentation of
//! `GCode.cpp` output-neutral twice): both sides see identical
//! C/P/N/angle/vec/norm at the call; the FIRST divergent operation is
//! the wipe offset distance. Upstream `GCode.cpp:5859`
//! `dist = std::min(scaled(nozzle_diam) * 0.5, scaled(width) * 0.5)`
//! with `scaled<coord_t>` (`Point.hpp:662-668`) dividing in the
//! OPERAND'S OWN float domain — the f32 width by f32(SCALING_FACTOR),
//! the f64 nozzle by f64(SCALING_FACTOR) — then truncating each to
//! coord_t: width 0.39999938011169434 → 399999 → 199999.5. Ares
//! `loop_paths.rs` used a continuous f64 `0.5·min(nozzle,width)/factor`
//! = 199999.69005584717; the 0.19-unit difference crosses the base
//! truncation boundary and lands native's pt_y exactly on the %.3f
//! half boundary 105.8915 (rounds up to .892) while Ares prints .891.
//!
//! GREEN (approved scope): reproduce the two typed `scaled` truncations
//! and the min-then-halve literally in `append_wipe_before_external`;
//! nothing else changes.

use crate::{OrcaRunner, golden, runner::stages};
use sha2::{Digest, Sha256};
use std::fs;

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_case(name: &str, expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let fixture = crate::runner::repo_root()
        .join("tests/parity/kobra-wipe-before-external")
        .join(format!("{name}.3mf"));
    let bytes = fs::read(&fixture).unwrap();
    assert_eq!(sha(&bytes), expected_sha, "fixture digest {name}");
    let root = tempfile::Builder::new()
        .prefix("kobra-wipe-before-external-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result = serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join("native/result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {name} {root:?}");
    let native = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let output = root.join("ares.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "CLI admission {name} {root:?}: {cli:?}"
    );
    let cli_bytes = fs::read(output).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("public API admission {name} {root:?}: {error:?}"));
    golden::compare_ordered_bytes_generator_only(&native, &api)
        .unwrap_or_else(|error| panic!("API strict comparison {name} {root:?}: {error}"));
    golden::compare_ordered_bytes_generator_only(&native, &cli_bytes)
        .unwrap_or_else(|error| panic!("CLI strict comparison {name} {root:?}: {error}"));
    assert_eq!(
        api, cli_bytes,
        "public API/CLI raw equality {name} {root:?}"
    );
}

#[test]
fn kobra_wipe_before_external_red() {
    run_case(
        "kobra-witness",
        "c8a16b7e4a95ed450144812890f6fef84a1039010d0036470b55fc3af550de62",
    );
}
