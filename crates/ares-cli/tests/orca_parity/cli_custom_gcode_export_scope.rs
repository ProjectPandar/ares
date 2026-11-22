#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    process::{Command, Output},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};

use crate::OrcaRunner;

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");
const START: &str =
    "{global scope_probe = 2; scope_probe = scope_probe + 1}\n; scope_start={scope_probe}";
const LAYER_READ: &str = "; scope_before={scope_probe}";
const LAYER_STEP: &str = "{if scope_probe < 7}\n; scope_before={scope_probe}\n{global scope_probe = scope_probe; scope_probe = scope_probe + 2}\n; scope_after={scope_probe}\n{endif}";
const END: &str = "; scope_end={scope_probe}";

fn project(templates: (&str, &str, &str)) -> Vec<u8> {
    let (start, layer, end) = templates;
    let mut input = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["machine_start_gcode"] = json!(start);
            settings["before_layer_change_gcode"] = json!("G92 E0");
            settings["layer_change_gcode"] = json!(layer);
            settings["machine_end_gcode"] = json!(end);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn slice_pair(
    label: &str,
    templates: (&str, &str, &str),
    expected_exit: i32,
) -> Option<[(Output, Option<String>); 2]> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let directory = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(runner.work_dir())
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    let out = directory.join("out");
    fs::write(&input, project(templates)).unwrap();
    fs::create_dir(&out).unwrap();
    let args = ["--slice", "1", "--outputdir", "out", "input.3mf"];
    let results = [
        ("native", Command::new(runner.bin())),
        ("ares", Command::new(env!("CARGO_BIN_EXE_ares"))),
    ]
    .map(|(side, mut command)| {
        let result = command.current_dir(&directory).args(args).output().unwrap();
        fs::write(directory.join(format!("{side}.stdout")), &result.stdout).unwrap();
        fs::write(directory.join(format!("{side}.stderr")), &result.stderr).unwrap();
        fs::write(
            directory.join(format!("{side}.exit")),
            format!("{:?}\n", result.status.code()),
        )
        .unwrap();
        let gcode = out.join("plate_1.gcode");
        let gcode = if gcode.exists() {
            let saved = directory.join(format!("{side}.gcode"));
            fs::rename(gcode, &saved).unwrap();
            Some(fs::read_to_string(saved).unwrap())
        } else {
            None
        };
        let result_json = out.join("result.json");
        if result_json.exists() {
            fs::rename(result_json, directory.join(format!("{side}.result.json"))).unwrap();
        }
        (result, gcode)
    });
    for (result, _) in &results {
        assert_eq!(result.status.code(), Some(expected_exit), "{directory:?}");
    }
    Some(results)
}

fn markers(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with("; scope_"))
        .collect()
}

#[test]
fn custom_gcode_globals_are_shared_from_start_through_layers_and_end() {
    let Some([(_, native), (_, ares)]) = slice_pair("export-globals", (START, LAYER_STEP, END), 0)
    else {
        return;
    };
    let native = native.unwrap();
    let ares = ares.unwrap();
    assert_eq!(
        markers(&native),
        [
            "; scope_start=3",
            "; scope_before=3",
            "; scope_after=5",
            "; scope_before=5",
            "; scope_after=7",
            "; scope_end=7",
        ]
    );
    assert_eq!(markers(&ares), markers(&native));
}

fn assert_layer_failure(label: &str, (start, layer): (&str, &str), message: &str) {
    let Some(results) = slice_pair(label, (start, layer, ""), 156) else {
        return;
    };
    for (result, gcode) in results {
        assert!(gcode.is_none());
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(text.contains(&format!(
            "layer_change_gcode Parsing error at line 1: {message}"
        )));
    }
}

#[test]
fn custom_gcode_locals_are_not_visible_in_later_templates() {
    assert_layer_failure(
        "export-locals",
        (&START.replacen("global", "local", 1), LAYER_READ),
        "Not a variable name",
    );
}

#[test]
fn custom_gcode_global_cannot_be_redeclared_as_local_in_a_later_template() {
    assert_layer_failure(
        "export-scope-collision",
        (START, "{local scope_probe = 1}"),
        "Variable name already defined in global scope",
    );
}

#[tokio::test]
async fn custom_gcode_globals_are_not_visible_in_an_independent_export() {
    async fn slice(input: &[u8]) -> Result<Vec<u8>, ares_core::SliceError> {
        ares_core::slice_project_plate_validated(
            input,
            GenerationMetadata::default(),
            Some(1),
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        )
        .await
    }

    let first = slice(&project((START, LAYER_STEP, END))).await.unwrap();
    let first = String::from_utf8(first).unwrap();
    assert!(markers(&first).contains(&"; scope_end=7"));

    let error = slice(&project(("", LAYER_READ, ""))).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("layer_change_gcode Parsing error at line 1: Not a variable name")
    );
}
