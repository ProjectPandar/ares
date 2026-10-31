//! Config validation acceptance controls, before the O2 loading migration.
use super::{OrcaRunner, stages, vendor, write_json};
use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest, SliceOptions};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[tokio::test]
async fn legacy_firmware_wipe_valid_pairs() {
    for (firmware, wipe) in [(false, true), (true, false)] {
        let directory = tempfile::Builder::new()
            .prefix("o2-valid-legacy-")
            .tempdir()
            .unwrap()
            .keep();
        let input = fs::read(super::super::repo_root().join("tests/parity/cube10.stl")).unwrap();
        fs::write(directory.join("cube10.stl"), &input).unwrap();
        let options_json = json!({"use_firmware_retraction": firmware, "wipe": wipe, "use_relative_e_distances": false});
        write_json(&directory.join("options.json"), &options_json);
        let native = OrcaRunner::from_env().unwrap().map(|runner| {
            stages::run(
                runner.bin(),
                [
                    format!("--use-firmware-retraction={}", u8::from(firmware)),
                    format!("--wipe={}", u8::from(wipe)),
                    "--use-relative-e-distances=0".to_owned(),
                    "--arrange".to_owned(),
                    "1".to_owned(),
                    "--export-3mf".to_owned(),
                    directory.join("native.3mf").to_str().unwrap().to_owned(),
                    directory.join("cube10.stl").to_str().unwrap().to_owned(),
                ],
                stages::Stage::Export,
                &directory,
            )
            .unwrap();
            assert!(directory.join("native.3mf").exists());
            "EXPORTED"
        });
        let options: SliceOptions = serde_json::from_value(options_json).unwrap();
        let actual = ares_core::slice_stl_validated(&input, options).await;
        write_json(
            &directory.join("result.json"),
            &json!({"native": native, "ares": actual.as_ref().map(|gcode| gcode.len()).map_err(ToString::to_string)}),
        );
        fs::write(directory.join("ares.gcode"), actual.unwrap()).unwrap();
    }
}

#[test]
fn profile_overrides_apply_before_validation() {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "RH3D").unwrap();
    let catalog_json = serde_json::to_string(&json!({"vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "RH3D")], "imports": []})).unwrap();
    let machine = "E3NG v1.2S - 0.4 nozzle";
    let process = "0.20mm Standard @E3NG v1.2S";
    let filament = "Generic PLA @E3NG v1.2S";
    let selection_json = serde_json::to_string(
        &json!({"machine": machine, "process": process, "filaments": [filament]}),
    )
    .unwrap();
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    for edits in [
        json!({"use_firmware_retraction": "0"}),
        json!({"wipe": ["0"]}),
    ] {
        let directory = tempfile::Builder::new()
            .prefix("o2-valid-profile-")
            .tempdir()
            .unwrap()
            .keep();
        fs::write(directory.join("cube10.stl"), &input).unwrap();
        write_json(&directory.join("overrides.json"), &edits);
        let mut machine_fields = profiles.machine(machine).unwrap();
        machine_fields.extend(edits.as_object().unwrap().clone());
        for (kind, fields) in [
            ("machine", machine_fields),
            ("process", profiles.process(process).unwrap()),
            ("filament", profiles.filament(filament).unwrap()),
        ] {
            write_json(&directory.join(format!("{kind}.json")), &json!(fields));
        }
        let native = OrcaRunner::from_env().unwrap().map(|runner| {
            let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
            stages::run(
                runner.bin(),
                [
                    "--load-settings".to_owned(),
                    format!("{};{}", path("machine.json"), path("process.json")),
                    "--load-filaments".to_owned(),
                    path("filament.json"),
                    "--arrange".to_owned(),
                    "1".to_owned(),
                    "--export-3mf".to_owned(),
                    path("native.3mf"),
                    path("cube10.stl"),
                ],
                stages::Stage::Export,
                &directory,
            )
            .unwrap();
            assert!(directory.join("native.3mf").exists());
            "EXPORTED"
        });
        let overrides = ConfigOverrides::from_json(&serde_json::to_vec(&edits).unwrap()).unwrap();
        let request = ProfileSliceRequest {
            filename: "cube10.stl",
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        };
        let actual = ares_core::validate_with_profiles(&input, request);
        write_json(
            &directory.join("result.json"),
            &json!({"native": native, "ares": actual.as_ref().map_err(ToString::to_string)}),
        );
        actual.unwrap();
        if let Some(runner) = OrcaRunner::from_env().unwrap() {
            let embedded =
                invalid_firmware_wipe_project(fs::read(directory.join("native.3mf")).unwrap());
            fs::write(directory.join("embedded-invalid.3mf"), &embedded).unwrap();
            let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
            stages::run(
                runner.bin(),
                [
                    "--load-settings".to_owned(),
                    format!("{};{}", path("machine.json"), path("process.json")),
                    "--load-filaments".to_owned(),
                    path("filament.json"),
                    "--export-3mf".to_owned(),
                    path("repaired.3mf"),
                    path("embedded-invalid.3mf"),
                ],
                stages::Stage::Export,
                &directory,
            )
            .unwrap();
            let actual = ares_core::validate_with_profiles(
                &embedded,
                ProfileSliceRequest {
                    filename: "embedded-invalid.3mf",
                    ..request
                },
            );
            write_json(
                &directory.join("embedded-result.json"),
                &json!({"native": "EXPORTED", "ares": actual.as_ref().map_err(ToString::to_string)}),
            );
            actual.unwrap();
        }
    }
}

fn invalid_firmware_wipe_project(exported: Vec<u8>) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(exported)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["use_firmware_retraction"] = json!("1");
            settings["wipe"] = json!(["1"]);
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
