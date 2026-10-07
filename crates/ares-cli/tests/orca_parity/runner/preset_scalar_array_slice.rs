//! Config.cpp:945-1001 JSON arrays -> scalar ConfigOptionFloat (Config.hpp:784).
//! Native CLI first; Ares receives the unflattened inherited user preset.
use std::fs;

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

#[tokio::test]
async fn orca_parity_inherited_user_process_scalar_array_whole_file() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "BBL").unwrap();
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let parent = "0.20mm Standard @BBL X1C";
    let user = "0.20mm Standard @BBL X1C - Mine";
    let filament = "Bambu PLA Basic @BBL X1C";
    let input = include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    let path = |name: &str| runner.work.join(name).to_str().unwrap().to_owned();
    fs::write(path("input.3mf"), input).unwrap();
    for (kind, fields) in [
        ("machine", profiles.machine(machine).unwrap()),
        ("filament", profiles.filament(filament).unwrap()),
    ] {
        write_json(
            &runner.work.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let cases = [
        ("array", json!(["50", "50"])),
        ("different", json!(["50", "60"])),
        ("single", json!(["50"])),
    ];
    let mut references = Vec::new();
    for (label, values) in &cases {
        let mut flattened = profiles.process(parent).unwrap();
        flattened.insert("name".into(), json!(user));
        flattened.insert("inherits".into(), json!(parent));
        flattened.insert("from".into(), json!("user"));
        flattened.insert("bridge_speed".into(), values.clone());
        write_json(
            &runner.work.join(format!("process-{label}.json")),
            &Value::Object(flattened),
        );
        let output = runner.work.join(format!("native-{label}"));
        fs::create_dir(&output).unwrap();
        stages::run(
            runner.bin(),
            [
                "--load-settings".to_owned(),
                format!(
                    "{};{}",
                    path("machine.json"),
                    path(&format!("process-{label}.json"))
                ),
                "--load-filaments".to_owned(),
                path("filament.json"),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path("input.3mf"),
            ],
            stages::Stage::Slice,
            &runner.work,
        )
        .unwrap();
        references.push(fs::read(output.join("plate_1.gcode")).unwrap());
    }
    let normalized_single = crate::golden::normalize_one_generator_line(
        &references[2],
        crate::golden::GeneratorKind::Orca,
    )
    .unwrap();
    for reference in &references[..2] {
        let normalized = crate::golden::normalize_one_generator_line(
            reference,
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap();
        assert!(
            crate::golden::first_difference(&normalized_single, &normalized).is_none(),
            "native array differs from single-element control; evidence {:?}",
            runner.work
        );
    }
    let selection = json!({"machine": machine, "process": user, "filaments": [filament]});
    write_json(&runner.work.join("selection.json"), &selection);
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let mut failures = Vec::new();
    for ((label, values), reference) in cases.iter().zip(&references).take(2) {
        let catalog = json!({
            "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "BBL")],
            "imports": [{"profiles": [{"type": "process", "name": user, "inherits": parent,
                "from": "user", "instantiation": "true", "bridge_speed": values}]}],
        });
        write_json(&runner.work.join(format!("catalog-{label}.json")), &catalog);
        let catalog_json = catalog.to_string();
        let actual = ares_core::slice_with_profiles(
            input,
            ProfileSliceRequest {
                filename: "input.3mf",
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
            &json!({
                "error": actual.as_ref().err().map(ToString::to_string),
                "native": "50,50 and 50,60 match single-element 50 (generated-by only)",
            }),
        );
        match actual {
            Ok(actual) => {
                fs::write(path(&format!("ares-{label}.gcode")), &actual).unwrap();
                if let Err(difference) =
                    crate::golden::compare_ordered_bytes_generator_only(reference, &actual)
                {
                    failures.push(format!("{label}: {difference}"));
                }
            }
            Err(error) => failures.push(format!("{label}: {error}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{failures:?}; evidence {:?}",
        runner.work
    );
}
