//! Config.cpp:945-1001 joins JSON arrays before ConfigOptionFloat reads a
//! numeric prefix (Config.hpp:784). Exercise the inherited user-preset path
//! with the existing LH whole-file control and matching raw vendor/user inputs.
use std::fs;

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::json;

use super::{OrcaRunner, stages, vendor, write_json};

#[tokio::test(flavor = "current_thread")]
async fn inherited_user_process_scalar_array_whole_file() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles_root = super::super::repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine = "LH Stinger 0.4 nozzle";
    let parent = profiles.compatible_process(machine).unwrap();
    let filament = profiles.compatible_filament(machine).unwrap();
    let user = format!("{parent} - Mine");
    let vendors = json!([
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, "LH"),
    ]);
    let input = super::placed_cube(include_bytes!("../../../../../tests/parity/cube10.stl"));
    let path = |name: &str| runner.work.join(name).to_str().unwrap().to_owned();
    fs::write(path("cube10.stl"), &input).unwrap();
    let selection = json!({"machine": machine, "process": user, "filaments": [filament]});
    let cases = [
        ("array", json!(["50", "50"])),
        ("different", json!(["50", "60"])),
        ("single", json!(["50"])),
    ];
    let mut references = Vec::new();
    for (label, values) in &cases {
        let catalog = json!({
            "vendors": vendors,
            "imports": [{"profiles": [{"type": "process", "name": user, "inherits": parent,
                "from": "user", "instantiation": "true", "bridge_speed": values,
                "initial_layer_print_height": ["50%", "60%"]}]}],
        });
        let output = runner.work.join(format!("native-{label}"));
        fs::create_dir(&output).unwrap();
        let native_presets =
            super::native_user_presets::resolved(&output, &profiles_root, &catalog, &selection);
        stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                native_presets.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                super::super::join_paths(native_presets.settings.iter()),
                "--load-filaments".to_owned(),
                super::super::join_paths(native_presets.filaments.iter()),
                "--arrange".to_owned(),
                "0".to_owned(),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path("cube10.stl"),
            ],
            stages::Stage::Slice,
            &runner.work,
        )
        .unwrap();
        references.push(fs::read(output.join("plate_1.gcode")).unwrap());
    }
    let normalize = |bytes: &[u8]| {
        crate::golden::normalize_one_generator_line(bytes, crate::golden::GeneratorKind::Orca)
            .unwrap()
    };
    let normalized_single = normalize(&references[2]);
    for reference in &references[..2] {
        assert!(
            crate::golden::first_difference(&normalized_single, &normalize(reference)).is_none(),
            "native array differs from single-element control; evidence {:?}",
            runner.work
        );
    }
    write_json(&runner.work.join("selection.json"), &selection);
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let mut failures = Vec::new();
    for ((label, values), reference) in cases.iter().zip(&references).take(2) {
        let catalog = json!({
            "vendors": vendors,
            "imports": [{"profiles": [{"type": "process", "name": user, "inherits": parent,
                "from": "user", "instantiation": "true", "bridge_speed": values,
                "initial_layer_print_height": ["50%", "60%"]}]}],
        });
        write_json(&runner.work.join(format!("catalog-{label}.json")), &catalog);
        let catalog_json = catalog.to_string();
        let actual = ares_core::slice_with_profiles(
            &input,
            ProfileSliceRequest {
                filename: "cube10.stl",
                catalog_json: &catalog_json,
                selection_json: &selection_json,
                plate: None,
                overrides: &overrides,
                placement: Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        )
        .await;
        write_json(
            &runner.work.join(format!("ares-result-{label}.json")),
            &json!({"error": actual.as_ref().err().map(ToString::to_string)}),
        );
        let actual = match actual {
            Ok(actual) => actual,
            Err(error) => {
                failures.push(format!("{label}: {error}"));
                continue;
            }
        };
        fs::write(path(&format!("ares-{label}.gcode")), &actual).unwrap();
        if let Err(difference) =
            crate::golden::compare_ordered_bytes_generator_only(reference, &actual)
        {
            failures.push(format!("{label}: {difference}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{failures:?}; evidence {:?}",
        runner.work
    );
}
