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
async fn outward_import_reaches_profile_slicing() {
    compare_slice("outward-ascii").await;
}

#[tokio::test]
async fn inward_import_reaches_profile_slicing() {
    compare_slice("inward-ascii").await;
}

#[tokio::test]
async fn wrong_authored_import_reaches_profile_slicing() {
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
            .join(format!("{case}.stl")),
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
        .args(["--slice", "0", "--outputdir"])
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
            placement: Placement::Center,
            sliced_plate: &mut plate,
        },
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    let expected = String::from_utf8(expected).unwrap();
    let actual = String::from_utf8(actual).unwrap();
    let native_z: Vec<_> = expected
        .lines()
        .filter_map(|line| line.strip_prefix(";Z:"))
        .collect();
    let ares_z: Vec<_> = actual
        .lines()
        .filter_map(|line| line.strip_prefix(";Z:"))
        .collect();
    let has_extrusion = |gcode: &str| {
        gcode.lines().any(|line| {
            line.starts_with("G1 ")
                && (line.contains(" X") || line.contains(" Y"))
                && line.contains(" E")
        })
    };
    fs::write(directory.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "case": case,
        "native_layer_z": native_z,
        "ares_layer_z": ares_z,
        "native_xy_extrusion": has_extrusion(&expected),
        "ares_xy_extrusion": has_extrusion(&actual),
        "scope": "public profile slice admission, layer Z sequence and XY extrusion presence only; not whole-G-code or XY path parity",
    })).unwrap()).unwrap();
    assert!(!native_z.is_empty());
    assert!(has_extrusion(&expected));
    assert_eq!(ares_z, native_z);
    assert_eq!(has_extrusion(&actual), has_extrusion(&expected));
}
