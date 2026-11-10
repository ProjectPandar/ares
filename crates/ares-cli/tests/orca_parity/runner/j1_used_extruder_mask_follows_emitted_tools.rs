//! `GCode.cpp:2901-2904`: by-layer usage follows emitted tools, not model defaults.

use super::{ExportedCase, OrcaRunner, cache_identity};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn j1_compatible_expression_groups_match_native() {
    run_compatible_expression_group_case(
        None,
        2,
        false,
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
    );
}

#[test]
fn j1_compatible_expression_groups_preserve_nonempty_filament_conditions() {
    run_compatible_expression_group_case(
        Some("J1_NONEMPTY_CONDITION".to_owned()),
        2,
        false,
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
    );
}

#[test]
fn j1_compatible_expression_groups_extend_existing_short_vectors() {
    let project = with_short_expression_groups(include_bytes!(
        "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
    ));
    run_compatible_expression_group_case(
        Some("J1_SHORT_VECTOR_CONDITION".to_owned()),
        3,
        true,
        &project,
    );
}

fn run_compatible_expression_group_case(
    condition: Option<String>,
    filament_count: usize,
    expect_native_rejection: bool,
    project: &[u8],
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-compatible-expression-groups-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output_dir = directory.join("native");
    fs::write(&input_path, project).unwrap();
    fs::create_dir(&output_dir).unwrap();
    let filament_source = include_bytes!(
        "../../../../../tests/parity/validation/load-settings-3mf/compatible-filament-0.json"
    );
    let mut filament = serde_json::from_slice::<serde_json::Value>(filament_source).unwrap();
    if let Some(condition) = condition.as_deref() {
        filament["compatible_printers_condition"] = condition.into();
        filament["compatible_prints_condition"] = condition.into();
    }
    let filament_paths = (0..filament_count)
        .map(|index| {
            let path = directory.join(format!("filament-{index}.json"));
            let mut filament = filament.clone();
            filament["name"] = format!("J1 Filament {index}").into();
            filament["filament_id"] = format!("j1-filament-{index}").into();
            fs::write(&path, serde_json::to_vec(&filament).unwrap()).unwrap();
            path
        })
        .collect::<Vec<_>>();
    let filament_arg = filament_paths
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(";");
    let native_result = crate::runner::stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--load-filaments".to_owned(),
            filament_arg.clone(),
            "--outputdir".to_owned(),
            output_dir.to_string_lossy().into_owned(),
            input_path.to_string_lossy().into_owned(),
        ],
        crate::runner::stages::Stage::Slice,
        &directory,
    );
    if expect_native_rejection {
        assert!(native_result.is_err());
    } else {
        native_result.unwrap();
    }
    let native =
        (!expect_native_rejection).then(|| fs::read(output_dir.join("plate_1.gcode")).unwrap());
    let ares_path = directory.join("ares.gcode");
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--load-filaments"])
        .arg(&filament_arg)
        .args(["-o"])
        .arg(&ares_path)
        .arg(&input_path)
        .env_remove("ARES_PARITY_PROJECT_PRESET_CATALOG")
        .output()
        .unwrap();
    assert_eq!(cli.status.success(), !expect_native_rejection);
    if expect_native_rejection {
        return;
    }
    let native = native.unwrap();
    let actual = fs::read(&ares_path).unwrap();
    let expected = compatible_expression_group_lines(&native);
    let actual_lines = compatible_expression_group_lines(&actual);
    fs::write(directory.join("native.gcode"), &native).unwrap();
    assert_eq!(actual_lines, expected, "see {directory:?}");
}

fn with_short_expression_groups(source: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["compatible_machine_expression_group"] =
                serde_json::json!(["RAW_MACHINE_0", "RAW_MACHINE_1"]);
            settings["compatible_process_expression_group"] = serde_json::json!(["RAW_PROCESS"]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compatible_expression_group_lines(bytes: &[u8]) -> Vec<String> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            line.starts_with("; compatible_machine_expression_group = ")
                || line.starts_with("; compatible_process_expression_group = ")
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn j1_custom_toolchange_fan_mover_prefix() {
    check_fan_mover_prefix(false);
}

#[test]
fn j1_custom_toolchange_fan_mover_disabled_prefix() {
    check_fan_mover_prefix(true);
}

fn check_fan_mover_prefix(disabled: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = include_bytes!(
        "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role2.3mf"
    );
    let project = if disabled {
        let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..input.len() {
            let mut file = input.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if file.name() == "Metadata/project_settings.config" {
                let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                settings["fan_speedup_time"] = "0".into();
                settings["fan_kickstart"] = "0".into();
                bytes = serde_json::to_vec(&settings).unwrap();
            }
            output
                .start_file(
                    file.name(),
                    zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
                )
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
        output.finish().unwrap().into_inner()
    } else {
        source.to_vec()
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-custom-toolchange-fan-mover-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output_dir = directory.join("native");
    fs::write(&input_path, &project).unwrap();
    fs::create_dir(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: format!("j1-custom-toolchange-fan-mover/disabled-{disabled}"),
            project,
            project_path: input_path,
            output_dir,
        })
        .unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    let expected = fan_mover_prefix(&sliced.reference);
    assert_eq!(expected.contains(";_FAN_MOVER_SKIP_START\n"), !disabled);
    assert_eq!(expected.contains(";_FAN_MOVER_SKIP_END\n"), !disabled);
    if disabled {
        assert!(
            !std::str::from_utf8(&sliced.reference)
                .unwrap()
                .contains(";_FAN_MOVER_SKIP_")
        );
    }
    let outcome = crate::compare_case(&sliced);
    let artifacts = outcome.artifacts.as_ref().unwrap();
    let actual = fs::read(artifacts.join("ares.gcode")).unwrap();
    fs::write(
        directory.join("prefix-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "disabled": disabled,
            "native_prefix": expected,
            "ares_prefix": fan_mover_prefix(&actual),
            "complete_status": outcome.status,
            "complete_detail": outcome.detail,
            "complete_artifacts": artifacts,
            "assertion_scope": "exact initial custom toolchange prefix; complete residual retained"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(fan_mover_prefix(&actual), expected, "{directory:?}");
    if disabled {
        assert!(
            !std::str::from_utf8(&actual)
                .unwrap()
                .contains(";_FAN_MOVER_SKIP_")
        );
    }
}

fn fan_mover_prefix(bytes: &[u8]) -> &str {
    let text = std::str::from_utf8(bytes).unwrap();
    let start = text
        .find("M83 ; use relative distances for extrusion\n")
        .unwrap();
    let rest = &text[start..];
    let end = rest.find(";_FORCE_RESUME_FAN_SPEED\n").unwrap() + ";_FORCE_RESUME_FAN_SPEED\n".len();
    &rest[..end]
}

fn check(name: &str, input: &[u8], sha256: &str, expected_usage: [bool; 2]) {
    assert_eq!(cache_identity::raw_digest(input), sha256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-used-extruder-follows-emitted-tools-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&input_path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: format!("j1-used-extruder-follows-emitted-tools/{name}"),
            project: input.to_vec(),
            project_path: input_path,
            output_dir: output,
        })
        .unwrap();
    let (actual, comparison) = if name == "role2" {
        let outcome = crate::compare_case(&sliced);
        let artifacts = outcome.artifacts.as_ref().unwrap();
        (
            fs::read(artifacts.join("ares.gcode")).unwrap(),
            Some((outcome.status, outcome.detail, artifacts.clone())),
        )
    } else {
        let actual = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(crate::runner::application::slice_project(
                input,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                ares_core::SlicerContext::new(0),
            ))
            .unwrap();
        fs::write(directory.join("ares.gcode"), &actual).unwrap();
        fs::write(directory.join("orca.gcode"), &sliced.reference).unwrap();
        (actual, None)
    };
    let native_witness = witness(&sliced.reference);
    let ares_witness = witness(&actual);
    fs::write(
        directory.join("usage-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": sha256,
            "native_sha256": cache_identity::raw_digest(&sliced.reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native": native_witness,
            "ares": ares_witness,
            "complete_comparison": comparison,
            "assertion_scope": if name == "role2" { "complete bytes; generated-by identity only; M73/footer retained" } else { "exact usage, initial and tool/flush witness only; known unrelated complete-byte residual retained separately" }
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_witness["usage"], serde_json::json!(expected_usage));
    let expected_initial = usize::from(name == "role2" || name == "both-used");
    assert_eq!(native_witness["initial_extruder"], expected_initial);
    assert_eq!(ares_witness["initial_extruder"], expected_initial);
    let expected_commands = match name {
        "role2" => vec!["T1", "T1", "G1 X324 E8.5437 F6840.0", "T1"],
        "both-used" => vec![
            "T1",
            "T0",
            "G1 X0 E8.5437 F6840.0",
            "T1",
            "G1 X324 E8.5437 F6840.0",
            "T1",
            "T0",
            "T1",
            "T0",
        ],
        "role1" | "by-object" => vec!["T0", "T0", "G1 X0 E8.5437 F6840.0", "T0"],
        _ => unreachable!(),
    };
    assert_eq!(
        native_witness["tool_and_flush_commands"],
        serde_json::json!(expected_commands)
    );
    if let Some((status, detail, _)) = comparison {
        assert_eq!(status, "PASS", "{detail}; {directory:?}");
    }
    assert_eq!(ares_witness, native_witness);
}

fn witness(bytes: &[u8]) -> serde_json::Value {
    let text = std::str::from_utf8(bytes).unwrap();
    let usage = [0, 1].map(|tool| {
        let prefix = format!("; --- T{tool}: ");
        text.lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap()
            .parse::<bool>()
            .unwrap()
    });
    let initial = text
        .lines()
        .find_map(|line| line.strip_prefix("; --- initial_extruder: "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let commands = text
        .lines()
        .filter(|line| {
            matches!(*line, "T0" | "T1")
                || line.starts_with("G1 X324 E8.5437")
                || line.starts_with("G1 X0 E8.5437")
        })
        .collect::<Vec<_>>();
    serde_json::json!({"usage": usage, "initial_extruder": initial, "tool_and_flush_commands": commands})
}

#[test]
fn j1_by_layer_role2_usage_matches_complete_output() {
    check(
        "role2",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role2.3mf"
        ),
        "7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4",
        [false, true],
    );
}

// Witness-only control: frozen whole-file first difference
// byte2449/line165 is Native M73 P41 versus Ares WIPE_START, unrelated to mask.
#[test]
fn j1_by_layer_role1_usage_witness() {
    check(
        "role1",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role1.3mf"
        ),
        "e74bf5ed81edcf4c00ea36be06967e1447ee4d61d25cd20b23fd39f16f6227e5",
        [true, false],
    );
}

// Frozen whole-file first difference byte2384/line189: Native FORCE_RESUME_FAN_SPEED
// versus Ares blank/change T0 -> T1. Only this control's exact witness is asserted.
#[test]
fn j1_by_layer_both_used_witness() {
    check(
        "both-used",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
        "12cf6020f118d5a2c5b5aa081f5dd9a4a3c043f0c3788558c13fa80687a4f3bd",
        [true, true],
    );
}

// Deferred by-object control: same unrelated byte2449/line165 residual as role1.
// Preserve exact usage/initial/tool witness; do not claim complete-byte parity.
#[test]
fn j1_by_object_usage_witness_unchanged() {
    check(
        "by-object",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/by-object.3mf"
        ),
        "828e4056bfdeaaed2f028fdbd596355a2f2fa56d0c8fab060f89268435823633",
        [true, false],
    );
}
