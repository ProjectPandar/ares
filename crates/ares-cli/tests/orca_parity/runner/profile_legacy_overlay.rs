//! Each loaded preset file normalizes through `set_deserialize`/`handle_legacy`
//! at load (`Config.cpp:577-579`, `PrintConfig.cpp:8033` renaming
//! `extruder_clearance_max_radius` at `:8248-8249`) **before**
//! `update_full_config` overlays it onto the embedded project
//! (`OrcaSlicer.cpp:2750-2812`). The BBL machine chain root
//! (`fdm_bbl_3dp_001_common`) carries the legacy
//! `extruder_clearance_max_radius = 68`, so the loaded machine replaces the
//! embedded Ender-3 `extruder_clearance_radius = 47` with `68`.

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use super::{
    OrcaRunner,
    profile_entry::{vendor, write_json},
    stages,
};
use crate::presets::VendorProfiles;

fn config_line<'a>(gcode: &'a str, key: &str) -> Option<&'a str> {
    gcode
        .lines()
        .find(|line| line.starts_with(&format!("; {key} = ")))
}

fn line_token(line: &str) -> &str {
    line.split_once(" = ").map(|(_, token)| token).unwrap_or("")
}

#[tokio::test]
async fn profile_legacy_machine_overlay_matches_native() {
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = VendorProfiles::load(&profiles_root, "BBL").unwrap();
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let process = "0.20mm Standard @BBL X1C";
    let filament = "Bambu PLA Basic @BBL X1C";
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "BBL")],
        "imports": [],
    });
    let selection = json!({"machine": machine, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("profile-legacy-overlay-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    for (kind, fields) in [
        ("machine", profiles.machine(machine).unwrap()),
        ("process", profiles.process(process).unwrap()),
        ("filament", profiles.filament(filament).unwrap()),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let input = include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    std::fs::write(directory.join("input.3mf"), input).unwrap();
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
    let ares = ares_core::slice_with_profiles(
        input,
        request,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let ares = String::from_utf8_lossy(&ares).into_owned();
    let ares_line = config_line(&ares, "extruder_clearance_radius")
        .expect("ares output lacks ; extruder_clearance_radius = line");
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
    let native_line = config_line(&native, "extruder_clearance_radius")
        .expect("native output lacks ; extruder_clearance_radius = line");
    assert_eq!(
        line_token(ares_line),
        line_token(native_line),
        "; extruder_clearance_radius mismatch (loaded machine legacy rename must precede the overlay)"
    );
    // The loaded machine chain's legacy value wins over the embedded one
    // (`PrintConfig.cpp:8248-8249` rename before `:2750-2812` overlay).
    assert_eq!(
        line_token(ares_line),
        "68",
        "extruder_clearance_radius must come from the loaded machine: {ares_line}"
    );
}
