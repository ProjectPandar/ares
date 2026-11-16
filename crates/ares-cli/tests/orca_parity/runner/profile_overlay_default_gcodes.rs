//! `OrcaSlicer.cpp:548-607 load_default_gcodes_to_config` creates every
//! printer G-code option on the loaded machine config (`:2850`,
//! `Preset::TYPE_PRINTER` — empty defaults except `machine_start_gcode` /
//! `machine_end_gcode`, PrintConfig.cpp:5947,2035) and one empty
//! `filament_start_gcode` / `filament_end_gcode` slot per loaded filament
//! (`:3150`). `update_full_config` then overlays them over the embedded 3MF
//! values, so a machine chain that never defines
//! `before_layer_change_gcode` replaces the embedded script with the empty
//! default (`OrcaSlicer.cpp:2750-2812`).

use ares_core::{ConfigOverrides, LoadSettingsFile, LoadSettingsSliceRequest};
use serde_json::Value;

use super::{OrcaRunner, profile_entry::write_json, stages};
use crate::presets::VendorProfiles;

/// The ten printer G-code keys `load_default_gcodes_to_config` creates
/// (`OrcaSlicer.cpp:546` gcodes key set).
const PRINTER_GCODE_KEYS: [&str; 10] = [
    "change_filament_gcode",
    "layer_change_gcode",
    "machine_end_gcode",
    "machine_pause_gcode",
    "machine_start_gcode",
    "template_custom_gcode",
    "printing_by_object_gcode",
    "before_layer_change_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
];

fn config_line<'a>(gcode: &'a str, key: &str) -> Option<&'a str> {
    gcode
        .lines()
        .find(|line| line.starts_with(&format!("; {key} = ")))
}

fn line_token(line: &str) -> &str {
    line.split_once(" = ").map(|(_, token)| token).unwrap_or("")
}

#[tokio::test]
async fn profile_overlay_default_gcodes() {
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = VendorProfiles::load(&profiles_root, "BBL").unwrap();
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let process = "0.20mm Standard @BBL X1C";
    let filament = "Bambu PLA Basic @BBL X1C";
    let directory = tempfile::Builder::new()
        .prefix("profile-default-gcodes-")
        .tempdir()
        .unwrap()
        .keep();
    let selected = [
        ("machine", profiles.machine(machine).unwrap()),
        ("process", profiles.process(process).unwrap()),
        ("filament", profiles.filament(filament).unwrap()),
    ];
    let settings_json = selected.map(|(kind, fields)| {
        let value = Value::Object(fields);
        write_json(&directory.join(format!("{kind}.json")), &value);
        value.to_string()
    });
    // The machine chain defines none of the three keys the embedded Ender-3
    // project supplies, so only the injected defaults may survive.
    let machine_fields = profiles.machine(machine).unwrap();
    for key in [
        "template_custom_gcode",
        "printing_by_object_gcode",
        "before_layer_change_gcode",
    ] {
        assert_eq!(
            machine_fields.get(key),
            None,
            "machine preset defines {key}"
        );
    }

    let input = include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    std::fs::write(directory.join("input.3mf"), input).unwrap();
    let settings = [
        LoadSettingsFile {
            label: "machine.json",
            json: &settings_json[0],
            source_identity: None,
        },
        LoadSettingsFile {
            label: "process.json",
            json: &settings_json[1],
            source_identity: None,
        },
    ];
    let filaments = [LoadSettingsFile {
        label: "filament.json",
        json: &settings_json[2],
        source_identity: None,
    }];
    let ares = ares_core::slice_with_load_settings(
        input,
        LoadSettingsSliceRequest {
            filename: "input.3mf",
            catalog_json: None,
            loaded_sources: None,
            uptodate: false,
            settings: &settings,
            filaments: &filaments,
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: ares_core::Placement::None,
            sliced_plate: &mut ares_core::SlicedPlate::default(),
        },
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let ares = String::from_utf8_lossy(&ares).into_owned();
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
    let output = directory.join("native");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), path("process.json")),
            "--load-filaments".to_owned(),
            path("filament.json"),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path("input.3mf"),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = std::fs::read_to_string(output.join("plate_1.gcode")).unwrap();
    for key in PRINTER_GCODE_KEYS {
        let native_line = config_line(&native, key)
            .unwrap_or_else(|| panic!("native output lacks ; {key} = line"));
        let ares_line =
            config_line(&ares, key).unwrap_or_else(|| panic!("ares output lacks ; {key} = line"));
        assert_eq!(
            line_token(ares_line),
            line_token(native_line),
            "; {key} mismatch"
        );
    }
    // The injected-default contract sits after the live-oracle comparison so
    // a regression fails on the native byte comparison first.
    for key in [
        "template_custom_gcode",
        "printing_by_object_gcode",
        "before_layer_change_gcode",
    ] {
        let line =
            config_line(&ares, key).unwrap_or_else(|| panic!("ares output lacks ; {key} = line"));
        assert_eq!(
            line_token(line),
            "",
            "injected default must replace the embedded value: {line}"
        );
    }
}
