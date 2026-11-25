use super::{MANIFEST, sha, witness};
use crate::runner::{OrcaRunner, stages};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct NativeCase {
    pub(super) root: PathBuf,
    pub(super) reference: Option<Vec<u8>>,
    pub(super) settings: Value,
    pub(super) contract: Value,
    pub(super) strict: bool,
    pub(super) native_stochastic_empty_layers: bool,
    pub(super) native_return_code: Option<i64>,
    pub(super) native_warning_message: Option<String>,
}

pub(super) fn prepare(
    runner: &OrcaRunner,
    label: &str,
    project: &[u8],
    seed: u32,
    root: &Path,
) -> NativeCase {
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    let contract = manifest["cases"][label].clone();
    assert_eq!(
        contract["preflight_only"], false,
        "preflight-only case is not parity evidence"
    );
    assert_eq!(sha(project), contract["input_sha256"].as_str().unwrap());
    let settings = crate::runner::application::settings(project).unwrap();
    for (key, expected) in contract["settings"].as_object().unwrap() {
        assert_eq!(&settings[key], expected, "input {key}");
    }
    let loaded = ares_core::load_project(project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 1);
    fs::create_dir_all(root.join("native")).unwrap();
    fs::write(root.join("input.3mf"), project).unwrap();
    fs::write(
        root.join("input.settings.json"),
        serde_json::to_vec_pretty(&settings).unwrap(),
    )
    .unwrap();
    let seeded = std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() == Ok("1");
    let strict = seeded || contract["official_contract"] == "strict";
    let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let seed_wire = seed.to_string();
    fs::write(
        root.join("native.launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(),"args":args,"native_binary":producer,
            "native_binary_sha256":sha(&fs::read(&producer).unwrap()),
            "input_sha256":sha(project),"seeded_replay":seeded,"seed":seed,
            "official_stochastic_is_diagnostic":!strict,
            "source":"FuzzySkin.cpp223-291,345-412,415-492,685-694; PerimeterGenerator.cpp378-387"
        }))
        .unwrap(),
    )
    .unwrap();
    let stochastic_empty_layers = match stages::run_live_with_env(
        runner.bin(),
        args,
        &[("ORCA_REPLAY_SEED", &seed_wire)],
        stages::Stage::Slice,
        root,
    ) {
        Ok(()) => false,
        Err(error) if !strict && label == "small-padding" => {
            let captured = error.captured.as_ref().unwrap();
            assert_eq!(captured.code, Some(156), "Native admission: {root:?}");
            let result: Value =
                serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap())
                    .unwrap();
            let warning = result["sliced_plates"][0]["warning_message"]
                .as_str()
                .unwrap();
            assert_eq!(result["return_code"], -100, "Native admission: {root:?}");
            assert!(
                warning.contains("The object has empty layers between 0.2 and 0.8"),
                "Native admission: {root:?}"
            );
            fs::write(
                root.join("native-stochastic-empty-layers.json"),
                serde_json::to_vec_pretty(&json!({
                    "return_code": result["return_code"],
                    "warning_message": warning,
                    "captured_code": captured.code,
                    "captured_status": captured.status,
                }))
                .unwrap(),
            )
            .unwrap();
            true
        }
        Err(error) => panic!("Native admission: {root:?}: {error:?}"),
    };
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    let (native_return_code, native_warning_message) = if stochastic_empty_layers {
        (
            Some(result["return_code"].as_i64().unwrap()),
            Some(
                result["sliced_plates"][0]["warning_message"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            ),
        )
    } else {
        assert_eq!(result["return_code"], 0, "Native admission: {root:?}");
        (None, None)
    };
    let reference =
        (!stochastic_empty_layers).then(|| fs::read(root.join("native/plate_1.gcode")).unwrap());
    let effective: serde_json::Map<String, Value> = reference
        .as_deref()
        .map(std::str::from_utf8)
        .transpose()
        .unwrap()
        .unwrap_or("")
        .lines()
        .filter_map(|line| {
            let (key, value) = line.strip_prefix("; ")?.split_once(" = ")?;
            Some((key.to_owned(), json!(value)))
        })
        .collect();
    if reference.is_some() {
        for key in [
            "wall_generator",
            "wall_loops",
            "fuzzy_skin",
            "fuzzy_skin_mode",
            "fuzzy_skin_noise_type",
            "fuzzy_skin_thickness",
            "fuzzy_skin_point_distance",
            "fuzzy_skin_first_layer",
            "fuzzy_skin_ripples_per_layer",
            "fuzzy_skin_ripple_offset",
            "fuzzy_skin_layers_between_ripple_offset",
        ] {
            assert_eq!(
                effective[key], settings[key],
                "Native effective {key}: {root:?}"
            );
        }
    }
    if let Some(reference) = &reference {
        fs::write(
            root.join("native.effective.json"),
            serde_json::to_vec_pretty(&effective).unwrap(),
        )
        .unwrap();
        let observed = witness::capture(reference);
        fs::write(
            root.join("native.wall-witness.json"),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        if !strict {
            witness::verify_diagnostic(&observed, &contract);
        }
    }
    NativeCase {
        root: root.to_owned(),
        reference,
        settings: Value::Object(settings),
        contract,
        strict,
        native_stochastic_empty_layers: stochastic_empty_layers,
        native_return_code,
        native_warning_message,
    }
}
