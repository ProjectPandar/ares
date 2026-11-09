//! PrintConfig.cpp:2043-2054 / Config.hpp:920-930: bridge-speed float-vector config.
use std::{
    fs,
    io::{Cursor, Read},
};

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

#[tokio::test]
async fn preset_scalar_original_bbl_array() {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "BBL").unwrap();
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let process = "0.20mm Standard @BBL X1C";
    let filament = "Bambu PLA Basic @BBL X1C";
    let alias = "My X1C 0.4 nozzle";
    let catalog = json!({"vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "BBL")], "imports": [{"profiles": [{"type": "machine", "name": alias, "inherits": machine, "from": "user", "instantiation": "true"}]}]});
    let selection = json!({"machine": alias, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("preset-scalar-bbl-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    fs::write(directory.join("input.3mf"), input).unwrap();
    let machine_fields = profiles.machine(machine).unwrap();
    let process_fields = profiles.process(process).unwrap();
    let bridge_speed = process_fields["bridge_speed"].clone();
    for (kind, fields) in [
        ("machine", machine_fields),
        ("process", process_fields),
        ("filament", profiles.filament(filament).unwrap()),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
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
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("input.3mf"),
            ],
            stages::Stage::Export,
            &directory,
        )
        .unwrap();
        let mut archive =
            zip::ZipArchive::new(Cursor::new(fs::read(directory.join("native.3mf")).unwrap()))
                .unwrap();
        let mut bytes = Vec::new();
        archive
            .by_name("Metadata/project_settings.config")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        let settings: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(settings["bridge_speed"], bridge_speed);
        json!({"status": "EXPORTED", "bridge_speed": settings["bridge_speed"]})
    });
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "input.3mf",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let loaded = ares_core::profile_settings(input, request).await;
    let placed = ares_core::profile_placed_project(input, request).await;
    write_json(
        &directory.join("result.json"),
        &json!({"native": native, "ares": loaded.as_ref().map(|view| json!(view)).map_err(ToString::to_string), "typed_bridge_speed": placed.as_ref().map(|project| &project.settings().process.region.bridge_speed.0).map_err(ToString::to_string)}),
    );
    let loaded = serde_json::to_value(
        loaded.unwrap_or_else(|error| panic!("{error}; evidence {directory:?}")),
    )
    .unwrap();
    assert_eq!(loaded["bridge_speed"], bridge_speed);
    let expected = bridge_speed
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            ares_core::Nullable::Value(ares_core::OrcaFloat(
                value.as_str().unwrap().parse().unwrap(),
            ))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placed.unwrap().settings().process.region.bridge_speed.0,
        expected
    );
}
