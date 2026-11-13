//! STL object-label identity: `ModelInstance::get_labeled_id`
//! (`Model.hpp:1433-1443`) falls back to the instance's `ObjectBase` id
//! while `loaded_id` stays 0 for STL inputs. Default volume members,
//! including eight texture-displacement facets (`Model.hpp:903-935`),
//! allocate through `ObjectID.hpp:86-87`; `GCode.cpp:4224-4239` emits the label.

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use super::{
    OrcaRunner,
    profile_entry::{placed_cube, vendor, write_json},
    stages,
};

#[tokio::test]
async fn stl_model_label_id() {
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
        .prefix("stl-label-id-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);

    let stl = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    // The oracle slices the same pre-positioned cube from the case directory.
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
    std::fs::write(directory.join("api.gcode"), &ares).unwrap();
    let ares = String::from_utf8_lossy(&ares).into_owned();
    let ares_line = ares
        .lines()
        .find(|line| line.starts_with("; model label id: "))
        .expect("ares output lacks ; model label id line");
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
    let native_line = native
        .lines()
        .find(|line| line.starts_with("; model label id: "))
        .expect("native output lacks ; model label id line");
    assert_eq!(ares_line, native_line, "; model label id mismatch");
}

#[test]
fn three_mf_instance_allocation_labels_match_native() {
    let root = super::repo_root().join("tests/parity/model-instance-label-allocation");
    let project = std::fs::read(root.join("build-allocation.3mf")).unwrap();
    let provenance: Value =
        serde_json::from_slice(&std::fs::read(root.join("provenance.json")).unwrap()).unwrap();
    assert_eq!(
        super::native_api_cli::sha(&project),
        provenance["input_sha256"].as_str().unwrap()
    );
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let native = super::native_api_cli::NativeCase::slice(
        &runner,
        &project,
        "three-mf-instance-allocation-labels-",
    );
    // Build allocation is 8,9,10,14; loaded37 wins for the third instance.
    // GCode.cpp:4233–4237 sorts the effective labels for the CSV header.
    assert_eq!(
        native
            .native
            .lines()
            .find(|line| line.starts_with("; model label id: "))
            .unwrap(),
        "; model label id: 8,9,14,37"
    );
    native.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}
