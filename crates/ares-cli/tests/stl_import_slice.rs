use std::{
    fs::{self, File},
    path::PathBuf,
    process::Command,
};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SlicedPlate, SlicerContext, slice_with_load_settings,
};
use serde_json::json;

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const MACHINE: &str = r#"{"type":"machine","name":"Acme A1 0.4 nozzle","from":"system","instantiation":"true","printer_model":"A1","printer_variant":"0.4","nozzle_diameter":["0.4"],"printable_area":["0x0","256x0","256x256","0x256"],"layer_change_gcode":"G92 E0"}"#;
const PROCESS: &str = r#"{"type":"process","name":"0.20mm Standard @A1","from":"system","instantiation":"true","layer_height":"0.2","compatible_printers":["Acme A1 0.4 nozzle"]}"#;
const FILAMENT: &str = r#"{"type":"filament","name":"Acme PLA","from":"system","instantiation":"true","filament_diameter":["1.75"],"filament_id":"AF000","filament_start_gcode":[""],"filament_end_gcode":[""]}"#;

#[tokio::test]
async fn outward_import_matches_native_gcode() {
    compare_slice("outward-ascii").await;
}

#[tokio::test]
async fn inward_import_matches_native_gcode() {
    compare_slice("inward-ascii").await;
}

#[tokio::test]
async fn wrong_authored_import_matches_native_gcode() {
    compare_slice("wrong-authored-ascii").await;
}

async fn compare_slice(case: &str) {
    let Some(native) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").unwrap());
    assert!(root.is_absolute());
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("stl-slice-{case}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("STL slice artifacts: {}", directory.display());
    let input = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/parity/stl-closed-import-repair")
            .join(format!("slice-{case}.stl")),
    )
    .unwrap();
    let model_path = directory.join(format!("{case}.stl"));
    fs::write(&model_path, &input).unwrap();
    for (label, text) in [
        ("machine.json", MACHINE),
        ("process.json", PROCESS),
        ("filament.json", FILAMENT),
    ] {
        fs::write(directory.join(label), text).unwrap();
    }
    let output = directory.join("output");
    fs::create_dir(&output).unwrap();
    let mut command = Command::new(native);
    command
        .args(["--debug", "5", "--datadir"])
        .arg(directory.join("data"))
        .arg("--load-settings")
        .arg(format!(
            "{};{}",
            directory.join("machine.json").display(),
            directory.join("process.json").display()
        ))
        .arg("--load-filaments")
        .arg(directory.join("filament.json"))
        .args(["--arrange", "0", "--slice", "0", "--outputdir"])
        .arg(&output)
        .arg(&model_path)
        .env("ORCA_JOB_APPDIR", directory.join("AppDir"))
        .env("OMP_NUM_THREADS", "1")
        .env("OPENBLAS_NUM_THREADS", "1")
        .current_dir(&directory)
        .stdout(File::create(directory.join("native.stdout")).unwrap())
        .stderr(File::create(directory.join("native.stderr")).unwrap());
    fs::write(
        directory.join("command.json"),
        serde_json::to_vec_pretty(&json!({
            "program": command.get_program().to_string_lossy(),
            "argv": command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
            "cwd": directory,
            "environment": {"ORCA_JOB_APPDIR": directory.join("AppDir"), "OMP_NUM_THREADS": "1", "OPENBLAS_NUM_THREADS": "1"},
        })).unwrap(),
    ).unwrap();
    let status = command.status().unwrap();
    fs::write(
        directory.join("native-status.json"),
        serde_json::to_vec_pretty(&json!({"exit_code":status.code(),"status":status.to_string()}))
            .unwrap(),
    )
    .unwrap();
    assert!(status.success(), "Native slicing failed: {status}");
    let expected = fs::read(output.join("plate_1.gcode")).unwrap();
    let mut plate = SlicedPlate::default();
    let actual = slice_with_load_settings(
        &input,
        LoadSettingsSliceRequest {
            orient: false,
            repetitions: 1,
            filename: &format!("{case}.stl"),
            catalog_json: None,
            uptodate: false,
            settings: &[
                LoadSettingsFile {
                    label: "machine.json",
                    json: MACHINE,
                },
                LoadSettingsFile {
                    label: "process.json",
                    json: PROCESS,
                },
            ],
            filaments: &[LoadSettingsFile {
                label: "filament.json",
                json: FILAMENT,
            }],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            sliced_plate: &mut plate,
        },
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    let expected = without_generator(&expected);
    let actual = without_generator(&actual);
    fs::write(directory.join("native.without-generator.gcode"), &expected).unwrap();
    fs::write(directory.join("ares.without-generator.gcode"), &actual).unwrap();
    let equal = actual == expected;
    let offset = expected
        .iter()
        .zip(&actual)
        .position(|(native, ares)| native != ares)
        .or_else(|| (expected.len() != actual.len()).then_some(expected.len().min(actual.len())));
    let first_difference = offset.map(|offset| {
        let line = expected[..offset].iter().filter(|&&byte| byte == b'\n').count() + 1;
        json!({
            "filtered_byte_offset": offset,
            "filtered_line": line,
            "native_line": expected.split(|&byte| byte == b'\n').nth(line - 1).map(String::from_utf8_lossy),
            "ares_line": actual.split(|&byte| byte == b'\n').nth(line - 1).map(String::from_utf8_lossy),
        })
    });
    fs::write(directory.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "case": case,
        "input_fixture": format!("slice-{case}.stl"),
        "native_arrange": 0,
        "ares_placement": "None",
        "whole_gcode_equal": equal,
        "first_difference": first_difference.as_ref(),
        "scope": "whole G-code bytes after removing exactly one generator line from each stream; no other normalization",
    })).unwrap()).unwrap();
    assert!(equal, "whole G-code differs: {first_difference:?}");
}

fn without_generator(gcode: &[u8]) -> Vec<u8> {
    let mut generator_lines = 0;
    let filtered = gcode
        .split_inclusive(|&byte| byte == b'\n')
        .filter(|line| {
            if line.starts_with(b"; generated by ") {
                generator_lines += 1;
                false
            } else {
                true
            }
        })
        .flatten()
        .copied()
        .collect();
    assert_eq!(generator_lines, 1);
    filtered
}
