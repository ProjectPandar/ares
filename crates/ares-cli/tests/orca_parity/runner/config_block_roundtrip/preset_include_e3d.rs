use super::super::{OrcaRunner, cache_identity, command, repo_root, stages};
use ares_core::{
    ConfigOverrides, Placement, PresetCatalogHandle, ProfileKind, ProfileSliceRequest,
    profile_settings,
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

const PRESET: &str = "Bambu PLA Basic @BBL X2D 0.4 nozzle";
const PROJECT: &str = "tests/parity/printer-variant-order/model-ironing-gyroid.3mf";
const CATALOG: &str = "tests/parity/preset-includes-e3d/catalog.json";

#[tokio::test]
async fn vendor_preset_includes_preserve_catalog_and_composition_variant_layout() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let project = fs::read(repo_root().join(PROJECT)).unwrap();
    let catalog = fs::read(repo_root().join(CATALOG)).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&project),
        "a17ab11edca5ef6ec93aa202d87f75e1898526a604c954dd166b939cc94b0553"
    );
    assert_eq!(
        cache_identity::raw_digest(&catalog),
        "afaf7cbdf9c481aa6a3c474f3f2046fdcc48645a22fd29b24524a5f9d08932f1"
    );
    fs::write(runner.work.join("catalog.json"), &catalog).unwrap();
    let identity = cache_identity::identify(&runner.bin).unwrap();
    fs::write(
        runner.work.join("native-producer.json"),
        serde_json::to_vec_pretty(&identity.receipt).unwrap(),
    )
    .unwrap();
    let mut exports = Vec::new();
    for replay in 0..2 {
        let work = runner.work.join(format!("native-export-{replay}"));
        fs::create_dir(&work).unwrap();
        let input = work.join("input.3mf");
        let output = work.join("settings.json");
        fs::write(&input, &project).unwrap();
        command::run(
            &runner.bin,
            [
                "--export-settings".to_owned(),
                output.to_string_lossy().into_owned(),
                input.to_string_lossy().into_owned(),
            ],
            &[("LC_ALL", "C")],
            command::Limits {
                stage: stages::Stage::Export,
                work: &work,
                timeout: Duration::from_secs(120),
            },
        )
        .unwrap();
        exports.push(fs::read(output).unwrap());
    }
    assert_eq!(
        exports[0], exports[1],
        "complete Native JSON replay differs"
    );
    let native: Value = serde_json::from_slice(&exports[0]).unwrap();
    let slots = native["filament_self_index"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| (slot == "1").then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(
        slots.len(),
        6,
        "Native did not resolve the qualified preset"
    );
    assert_eq!(native["filament_self_index"].as_array().unwrap().len(), 12);
    assert_eq!(native["filament_map"], json!(["1", "1"]));
    let variants = slots
        .iter()
        .map(|&index| native["filament_extruder_variant"][index].clone())
        .collect::<Vec<_>>();
    let retraction = slots
        .iter()
        .map(|&index| native["filament_retraction_length"][index].clone())
        .collect::<Vec<_>>();

    let envelope: Value = serde_json::from_slice(&catalog).unwrap();
    let mut handle = PresetCatalogHandle::new();
    handle
        .add_vendor(&serde_json::to_vec(&envelope["vendors"][0]).unwrap())
        .unwrap();
    let names = [PRESET.to_owned()];
    let keys = ["filament_extruder_variant", "filament_retraction_length"];
    let values = serde_json::to_value(
        handle
            .preset_values(ProfileKind::Filament, &names, &keys)
            .unwrap(),
    )
    .unwrap();
    let replay = serde_json::to_value(
        handle
            .preset_values(ProfileKind::Filament, &names, &keys)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(values, replay);
    fs::write(
        runner.work.join("catalog-values.json"),
        serde_json::to_vec_pretty(&values).unwrap(),
    )
    .unwrap();
    fs::write(
        runner.work.join("catalog-values-replay.json"),
        serde_json::to_vec_pretty(&replay).unwrap(),
    )
    .unwrap();

    let stl = fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap();
    let selection = json!({
        "machine": "Bambu Lab X2D 0.4 nozzle",
        "process": "0.20mm Standard @BBL X2D",
        "filaments": [PRESET],
    })
    .to_string();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: std::str::from_utf8(&catalog).unwrap(),
        selection_json: &selection,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let composition = serde_json::to_value(profile_settings(&stl, request).await.unwrap()).unwrap();
    let replay = serde_json::to_value(profile_settings(&stl, request).await.unwrap()).unwrap();
    assert_eq!(composition, replay);
    fs::write(
        runner.work.join("composition-settings.json"),
        serde_json::to_vec_pretty(&composition).unwrap(),
    )
    .unwrap();
    fs::write(
        runner.work.join("composition-settings-replay.json"),
        serde_json::to_vec_pretty(&replay).unwrap(),
    )
    .unwrap();
    assert_eq!(values[0]["filament_retraction_length"], json!(retraction));
    assert_eq!(composition["filament_retraction_length"], json!(retraction));

    let observed = json!({
        "catalog": values[0]["filament_extruder_variant"],
        "composition": composition["filament_extruder_variant"],
    });
    let expected = json!({"catalog": variants, "composition": variants});
    fs::write(
        runner.work.join("include-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "actual": observed, "expected_from_native": expected,
            "pass": observed == expected,
            "stl_input_sha256": cache_identity::raw_digest(&stl),
            "scope": "selected preset include resolution, not whole GCode parity",
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        observed, expected,
        "shared preset include resolution differs"
    );
}
