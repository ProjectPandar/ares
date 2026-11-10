//! Native-valid TiQ2 inputs precede the Print.cpp:1650-1733 rejection port.
use std::fs;

use crate::runner::{ExportedCase, OrcaRunner, application, cache_identity, stages};

use crate::print_validation::tiq2_corner::{Scenario, corner_input};

#[test]
fn tiq2_unchanged_estimated_tower_printable_rejection() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("estimated tower rejection needs Native");
        return;
    };
    let input = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(input),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let directory = tempfile::Builder::new()
        .prefix("tiq2-unchanged-tower-rejection-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), input).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(input)).unwrap();
    let settings: serde_json::Value =
        serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap())
            .unwrap();
    assert_eq!(settings["enable_wrapping_detection"], "0");
    assert_eq!(settings["wrapping_exclude_area"], serde_json::json!([]));
    assert_eq!(settings["bed_exclude_area"], serde_json::json!(["0x0"]));
    assert_eq!(settings["extruder_printable_area"], serde_json::json!([]));
    fs::write(
        directory.join("deferred-check-admission.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": cache_identity::raw_digest(input),
            "enable_wrapping_detection": settings["enable_wrapping_detection"],
            "wrapping_exclude_area": settings["wrapping_exclude_area"],
            "bed_exclude_area": settings["bed_exclude_area"],
            "extruder_printable_area": settings["extruder_printable_area"],
        }))
        .unwrap(),
    )
    .unwrap();
    fs::create_dir(directory.join("native")).unwrap();
    let error = stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            directory.join("native").to_string_lossy().into_owned(),
            directory.join("input.3mf").to_string_lossy().into_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    assert_eq!(error.kind, stages::FailureKind::Process);
    let captured = error.captured.unwrap();
    fs::write(directory.join("native.stdout"), &captured.stdout).unwrap();
    fs::write(directory.join("native.stderr"), &captured.stderr).unwrap();
    assert_eq!(
        captured.code,
        Some(if cfg!(unix) {
            (-64_i32).rem_euclid(256)
        } else {
            -64
        })
    );
    let diagnostic =
        "Prime tower is partially outside the printable area, and it cannot be printed.";
    assert_eq!(
        String::from_utf8(captured.stderr.clone()).unwrap(),
        format!("{diagnostic}\n\n")
    );
    let native_result: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(native_result["return_code"], -64);
    assert!(!directory.join("native/plate_1.gcode").exists());
    fs::create_dir(directory.join("ares")).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(directory.join("ares/api.gcode"), bytes).unwrap();
    }
    let output = directory.join("ares/plate_1.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares/stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares/stderr"), &cli.stderr).unwrap();
    let cli_raw = fs::read(directory.join("ares/result.json")).unwrap();
    let cli_result: serde_json::Value = serde_json::from_slice(&cli_raw).unwrap();
    let comparison = serde_json::json!({
        "input_sha256": cache_identity::raw_digest(input), "exit": cli.status.code(),
        "api_error": api.as_ref().err().map(|e| format!("{e:?}")),
        "api_gcode_bytes": api.as_ref().ok().map(Vec::len),
        "cli_gcode_bytes": fs::read(&output).ok().map(|b| b.len()),
        "result": cli_result, "native_result": native_result,
        "raw_result_equal": cli_raw == fs::read(directory.join("native/result.json")).unwrap(),
        "whole_output_parity_claim": false,
    });
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&comparison).unwrap(),
    )
    .unwrap();
    assert!(
        matches!(&api, Err(ares_core::SliceError::LayeredPrintCollision { message, .. }) if message == diagnostic),
        "unchanged input must fail estimated tower clearance; {}: {comparison}",
        directory.display()
    );
    assert_eq!(cli.status.code(), captured.code);
    assert_eq!(cli.stdout, captured.stdout);
    assert_eq!(cli.stderr, captured.stderr);
    assert_eq!(cli_result["return_code"], native_result["return_code"]);
    assert_eq!(cli_result["error_string"], native_result["error_string"]);
    assert!(!output.exists());
}

#[test]
fn tiq2_brim_corner_five_native_premises() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("TiQ2 corner premises need Native");
        return;
    };
    let original = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(original),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let root = tempfile::Builder::new()
        .prefix("tiq2-brim-corner-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    for scenario in [
        Scenario::RetainedFan,
        Scenario::WholeFile,
        Scenario::Slots,
        Scenario::Unused,
        Scenario::Assignment,
    ] {
        let directory = root.join(scenario.label());
        fs::create_dir_all(directory.join("native")).unwrap();
        let input = corner_input(original, scenario, &directory);
        let path = directory.join("input.3mf");
        fs::write(&path, &input).unwrap();
        // Force exactly one slice for each fixture, including byte-identical variants.
        let result = runner.rerun_reference(&ExportedCase {
            label: format!("TiQ2/brim-corner/{}", scenario.label()),
            project: input,
            project_path: path,
            output_dir: directory.join("native"),
        });
        match result {
            Ok(case) => {
                assert!(!case.reference.is_empty());
                fs::write(directory.join("native.gcode"), case.reference).unwrap();
                fs::write(
                    directory.join("native-success.json"),
                    b"{\"exported\":true}\n",
                )
                .unwrap();
            }
            Err(error) => {
                if let Some(captured) = &error.captured {
                    fs::write(directory.join("native.stdout"), &captured.stdout).unwrap();
                    fs::write(directory.join("native.stderr"), &captured.stderr).unwrap();
                    fs::write(
                        directory.join("native-rejection.json"),
                        serde_json::to_vec_pretty(&serde_json::json!({
                            "stage": format!("{:?}", error.stage),
                            "exit": captured.code,
                            "stdout": captured.stdout,
                            "stderr": captured.stderr,
                            "no_retry": true,
                            "no_ares_slice": true,
                            "no_port": true,
                        }))
                        .unwrap(),
                    )
                    .unwrap();
                }
                panic!(
                    "Native rejected {}: {error:?}; no alternate position, Ares or port; {}",
                    scenario.label(),
                    directory.display()
                );
            }
        }
    }
}
