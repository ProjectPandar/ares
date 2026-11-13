//! The harness flattener must produce the upstream merged preset:
//! `Preset.cpp:1680-1684` copies the parent config and applies
//! `update_diff_values_to_child_config` (`PrintConfig.cpp:10209-10298`),
//! which skips the variant key (`:10263`) and maps child vector values onto
//! the parent's variant layout, so the merged `Bambu PLA Basic @BBL X1C`
//! keeps one `filament_extruder_variant` slot and the CLI sizes
//! `filament_self_index` to one entry (`OrcaSlicer.cpp:3117-3337`). A key
//! overwrite flattener keeps the child's two variants and the native oracle
//! emits `1,1` against Ares' `1`.

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::json;

use super::{
    OrcaRunner,
    profile_entry::{placed_cube, vendor, write_json},
    stages,
};

fn config_line<'a>(gcode: &'a str, key: &str) -> Option<&'a str> {
    gcode
        .lines()
        .find(|line| line.starts_with(&format!("; {key} = ")))
}

fn line_token(line: &str) -> &str {
    line.split_once(" = ").map(|(_, token)| token).unwrap_or("")
}

#[tokio::test]
async fn filament_self_index_matches_native() {
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let process = "0.20mm Standard @BBL X1C";
    let filament = "Bambu PLA Basic @BBL X1C";
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "BBL")],
        "imports": [],
    });
    let selection = json!({"machine": machine, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("filament-self-index-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(directory.join("cube10.stl"), &stl).unwrap();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let ares = ares_core::slice_with_profiles(
        &stl,
        request,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let ares = String::from_utf8_lossy(&ares).into_owned();
    let ares_line = config_line(&ares, "filament_self_index")
        .expect("ares output lacks ; filament_self_index = line");
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let native_presets = super::profile_entry::native_user_presets::resolved(
        &directory,
        &profiles_root,
        &catalog,
        &selection,
    );
    let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
    let output = directory.join("native");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            native_presets.datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            super::join_paths(native_presets.settings.iter()),
            "--load-filaments".to_owned(),
            super::join_paths(native_presets.filaments.iter()),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path("cube10.stl"),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = std::fs::read_to_string(output.join("plate_1.gcode")).unwrap();
    let native_line = config_line(&native, "filament_self_index")
        .expect("native output lacks ; filament_self_index = line");
    assert_eq!(
        line_token(ares_line),
        line_token(native_line),
        "; filament_self_index mismatch (harness flatten must keep the parent variant layout)"
    );
    assert_eq!(
        line_token(ares_line),
        "1",
        "the merged filament preset carries one variant slot: {ares_line}"
    );
}
