use super::super::{OrcaRunner, command, repo_root, stages};
use ares_core::{
    ConfigOverrides, Placement, PresetCatalogHandle, ProfileKind, ProfileSliceRequest,
    profile_settings,
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

#[path = "preset_include_controls/inputs.rs"]
mod inputs;

async fn check(case: &str) -> Result<(), String> {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return Ok(());
    };
    let catalog = inputs::prepare(&runner.work, case);
    let output = runner.work.join("native-settings.json");
    command::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            runner.work.join("data").to_string_lossy().into_owned(),
            "--uptodate".to_owned(),
            "--export-settings".to_owned(),
            output.to_string_lossy().into_owned(),
            runner
                .work
                .join("control-input.3mf")
                .to_string_lossy()
                .into_owned(),
        ],
        &[("LC_ALL", "C")],
        command::Limits {
            stage: stages::Stage::Export,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap();
    let native: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let keys = [
        "filament_extruder_variant",
        "filament_max_volumetric_speed",
        "filament_flow_ratio",
    ];
    let mut handle = PresetCatalogHandle::new();
    for vendor in catalog["vendors"].as_array().unwrap() {
        handle
            .add_vendor(&serde_json::to_vec(vendor).unwrap())
            .unwrap();
    }
    let values = handle
        .preset_values(ProfileKind::Filament, &[inputs::PRESET.to_owned()], &keys)
        .map(|value| serde_json::to_value(value).unwrap())
        .map_err(|error| error.to_string());
    let input = fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap();
    let selection = json!({"machine": "Bambu Lab X2D 0.4 nozzle", "process": "0.20mm Standard @BBL X2D", "filaments": [inputs::PRESET]}).to_string();
    let catalog_json = catalog.to_string();
    let overrides = ConfigOverrides::default();
    let composed = profile_settings(
        &input,
        ProfileSliceRequest {
            filename: "cube10.stl",
            catalog_json: &catalog_json,
            selection_json: &selection,
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        },
    )
    .await
    .map(|value| serde_json::to_value(value).unwrap())
    .map_err(|error| error.to_string());
    fs::write(
        runner.work.join("api-results.json"),
        serde_json::to_vec_pretty(
            &json!({"case": case, "catalog": values, "composition": composed}),
        )
        .unwrap(),
    )
    .unwrap();
    if matches!(
        case,
        "missing-include" | "forward-include" | "cross-vendor-include"
    ) {
        assert!(
            fs::read_to_string(runner.work.join("00000.log"))
                .unwrap()
                .contains("can not find include")
        );
        assert_eq!(
            native["filament_extruder_variant"]
                .as_array()
                .unwrap()
                .len(),
            8,
            "Native did not retain the archive after source resolution failed"
        );
        if values.is_ok() || composed.is_ok() {
            return Err(format!(
                "{case}: catalog/composition accepted an unavailable Native include"
            ));
        }
    } else {
        let slots = native["filament_self_index"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| (slot == "1").then_some(i))
            .collect::<Vec<_>>();
        assert_eq!(slots.len(), 6);
        let values = values?;
        let composed = composed?;
        for key in keys {
            let expected = json!(
                slots
                    .iter()
                    .map(|&i| native[key][i].clone())
                    .collect::<Vec<_>>()
            );
            if values[0][key] != expected || composed[key] != expected {
                return Err(format!(
                    "{case}/{key}: expected {expected}, catalog {}, composition {}",
                    values[0][key], composed[key]
                ));
            }
        }
    }
    Ok(())
}

pub(super) async fn check_all() -> Vec<String> {
    let mut failures = Vec::new();
    for case in [
        "no-include",
        "ordered",
        "reverse",
        "leaf-precedence",
        "pre-padding-default",
        "missing-include",
        "forward-include",
        "cross-vendor-include",
    ] {
        if let Err(error) = check(case).await {
            failures.push(error);
        }
    }
    failures
}
