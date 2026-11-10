//! Timelapse configuration at the layer change (`GCode.cpp:6516-6550`, `:6710-6713`).
//!
//! A non-BBL printer outside the traditional gate renders `time_lapse_gcode` at the layer change
//! through `timelapse_gcode_config`, the same configuration every other render uses. Fixture:
//! stock MK4 `cube_with_hole` appending `;ARES_PROBE_TLMPE={most_used_physical_extruder_id}`.

#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
};

use serde_json::{Value, json};

use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, native_api_cli::NativeCase, repo_root, stages},
};

const FIXTURE: &str = "tests/parity/validation/cli-exit/custom-gcode/mk4-tlmpe.3mf";

fn fixture() -> Vec<u8> {
    fs::read(repo_root().join(FIXTURE)).unwrap()
}

fn with_settings(edits: &[(&str, Value)]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(fixture())).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in edits {
                settings[*key] = value.clone();
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn assert_one_render_per_layer(native: &str, marker: &str) {
    let lines = |prefix: &str| {
        native
            .lines()
            .filter(|line| line.starts_with(prefix))
            .collect::<Vec<_>>()
    };
    let layers = lines(";LAYER_CHANGE").len();
    assert!(layers > 0);
    let renders = lines(marker);
    assert_eq!(renders.len(), layers, "one render per layer");
    assert!(
        renders.iter().all(|line| !line.contains('{')),
        "Native resolves every variable: {renders:?}"
    );
}

#[test]
fn custom_gcode_timelapse_overlay_mk4_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = NativeCase::slice(&runner, &fixture(), "timelapse-overlay-mk4-");
    assert_one_render_per_layer(&case.native, ";ARES_PROBE_TLMPE=");
    case.compare(ares_core::GenerationMetadata::default());
}

#[test]
fn layer_change_template_resolves_every_timelapse_variable_like_native() {
    const MARKER: &str = ";TIMELAPSE_CONFIG";
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let template = [
        "layer_num",
        "layer_z",
        "max_layer_z",
        "most_used_physical_extruder_id",
        "curr_physical_extruder_id",
        "timelapse_pos_x",
        "timelapse_pos_y",
        "has_timelapse_safe_pos",
        "timelapse_inline_photo",
        "farthest_point_timelapse_enabled",
        "clear_to_x0",
    ]
    .iter()
    .fold(MARKER.to_owned(), |line, name| {
        format!("{line} {name}={{{name}}}")
    });
    for (timelapse_type, print_sequence) in [
        ("0", "by layer"),
        ("1", "by layer"),
        ("0", "by object"),
        ("1", "by object"),
    ] {
        let project = with_settings(&[
            ("time_lapse_gcode", json!(template)),
            ("timelapse_type", json!(timelapse_type)),
            ("print_sequence", json!(print_sequence)),
        ]);
        let case = NativeCase::slice(&runner, &project, "timelapse-config-variables-");
        for echoed in [
            format!("; timelapse_type = {timelapse_type}"),
            format!("; print_sequence = {print_sequence}"),
        ] {
            assert!(case.native.lines().any(|line| line == echoed), "{echoed}");
        }
        assert_one_render_per_layer(&case.native, MARKER);
        case.compare(ares_core::GenerationMetadata::default());
    }
}

/// `TimelapsePosPicker.hpp:83`, `.cpp:36`, `:361-363`: the camera limit inflates by the clearance
/// truncated to an int and halved by integer division.
#[test]
fn safe_position_follows_truncated_clearance_like_native() {
    const MARKER: &str = ";TIMELAPSE_POSITION";
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let template =
        format!("{MARKER} {{timelapse_pos_x}} {{timelapse_pos_y}} {{has_timelapse_safe_pos}}");
    for clearance in ["65", "67.5", "64.9", "71"] {
        let project = with_settings(&[
            ("time_lapse_gcode", json!(template)),
            ("extruder_clearance_radius", json!(clearance)),
        ]);
        let case = NativeCase::slice(&runner, &project, "timelapse-position-clearance-");
        let echoed = format!("; extruder_clearance_radius = {clearance}");
        assert!(case.native.lines().any(|line| line == echoed), "{echoed}");
        assert_one_render_per_layer(&case.native, MARKER);
        case.compare(ares_core::GenerationMetadata::default());
    }
}

#[test]
fn unknown_timelapse_variable_rejects_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("timelapse-unknown-variable-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = root.join("input.3mf");
    fs::write(
        &project,
        with_settings(&[(
            "time_lapse_gcode",
            json!(";UNKNOWN={no_such_timelapse_variable}"),
        )]),
    )
    .unwrap();
    let native_output = root.join("native");
    fs::create_dir(&native_output).unwrap();
    let failure = stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_output.to_str().unwrap().to_owned(),
            project.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &root,
    )
    .unwrap_err();
    let native = failure.captured.unwrap();
    let native_stderr = String::from_utf8_lossy(&native.stderr);
    assert!(
        native_stderr.contains("Not a variable name"),
        "{native_stderr}"
    );
    assert!(!native_output.join("plate_1.gcode").exists());

    let retained = root.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), native.code, "{root:?}");
    assert_eq!(output.stdout, native.stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), native_stderr);
    assert_eq!(
        result_comparison_bytes(&fs::read(root.join("result.json")).unwrap()),
        result_comparison_bytes(&fs::read(native_output.join("result.json")).unwrap()),
    );
    assert!(!retained.exists(), "no G-code on a template failure");
}
