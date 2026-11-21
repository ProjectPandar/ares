//! Preset.cpp:1925–1944 copies completed collection defaults through user inheritance.

use std::fs;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SlicerContext,
};
use serde_json::json;

use super::{OrcaRunner, native_user_presets, stages, vendor, write_json};

#[tokio::test]
async fn inherited_catalog_user_presets_preserve_collection_defaults() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles_root = super::super::repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine = "LH Stinger 0.4 nozzle";
    let process = profiles.compatible_process(machine).unwrap();
    let filament = profiles.compatible_filament(machine).unwrap();
    let vendors = json!([
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, "LH"),
    ]);
    let input = super::placed_cube(include_bytes!("../../../../../tests/parity/cube10.stl"));
    let input_path = runner.work.join("cube10.stl");
    fs::write(&input_path, &input).unwrap();
    let overrides = ConfigOverrides::default();
    let mut failures = Vec::new();
    for (kind, parent) in [
        ("process", process.as_str()),
        ("machine", machine),
        ("filament", filament.as_str()),
    ] {
        let user = format!("{parent} - Mine");
        let catalog = json!({
            "vendors": vendors,
            "imports": [{"profiles": [{"type": kind, "name": user, "inherits": parent,
                "from": "user", "instantiation": "true"}]}],
        });
        let mut selection =
            json!({"machine": machine, "process": process, "filaments": [filament]});
        if kind == "filament" {
            selection["filaments"][0] = json!(user);
        } else {
            selection[kind] = json!(user);
        }
        write_json(&runner.work.join(format!("catalog-{kind}.json")), &catalog);
        write_json(
            &runner.work.join(format!("selection-{kind}.json")),
            &selection,
        );
        let output = runner.work.join(format!("native-{kind}"));
        fs::create_dir(&output).unwrap();
        let native = native_user_presets::resolved(&output, &profiles_root, &catalog, &selection);
        stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                native.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                super::super::join_paths(native.settings.iter()),
                "--load-filaments".to_owned(),
                super::super::join_paths(native.filaments.iter()),
                "--arrange".to_owned(),
                "0".to_owned(),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                input_path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &runner.work,
        )
        .unwrap();
        let reference = fs::read(output.join("plate_1.gcode")).unwrap();
        let actual = ares_core::slice_with_profiles(
            &input,
            ProfileSliceRequest {
                filename: "cube10.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: Placement::None,
            },
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            SlicerContext::new(0),
        )
        .await;
        write_json(
            &runner.work.join(format!("ares-result-{kind}.json")),
            &json!({"error": actual.as_ref().err().map(ToString::to_string)}),
        );
        let actual = match actual {
            Ok(actual) => actual,
            Err(error) => {
                failures.push(format!("{kind}: {error}"));
                continue;
            }
        };
        fs::write(runner.work.join(format!("ares-{kind}.gcode")), &actual).unwrap();
        if let Err(difference) =
            crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        {
            failures.push(format!("{kind}: {difference}"));
        }
    }
    write_json(&runner.work.join("differences.json"), &json!(failures));
    assert!(
        failures.is_empty(),
        "{failures:?}; artifacts {:?}",
        runner.work
    );
}
