use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use crate::runner::{
    OrcaRunner, join_paths,
    profile_entry::{native_user_presets, vendor, write_json},
    repo_root, stages,
};

#[tokio::test]
async fn vendor_nil_child_uses_nonzero_parent_default() {
    compare(
        "filament_flush_temp",
        json!(["120"]),
        json!(["nil"]),
        json!(["120"]),
    )
    .await;
}

#[tokio::test]
async fn vendor_nonnil_child_keeps_authored_value() {
    compare(
        "filament_flush_temp",
        json!(["120"]),
        json!(["220"]),
        json!(["220"]),
    )
    .await;
}

#[tokio::test]
async fn vendor_override_nil_is_not_completed() {
    compare(
        "filament_retraction_length",
        json!(["0.7"]),
        json!(["nil"]),
        json!(["nil"]),
    )
    .await;
}

#[tokio::test]
async fn vendor_mixed_vector_is_not_wholly_nil() {
    compare(
        "filament_flush_temp",
        json!(["130"]),
        json!(["120", "nil"]),
        json!(["120"]),
    )
    .await;
}

#[tokio::test]
async fn vendor_nil_parent_is_completed_before_child() {
    compare(
        "filament_flush_temp",
        json!(["nil"]),
        json!(["nil"]),
        json!(["0"]),
    )
    .await;
}

#[test]
fn direct_sparse_nil_keeps_range_rejection() {
    let root = repo_root();
    let profiles =
        crate::presets::VendorProfiles::load(&root.join("OrcaSlicer/resources/profiles"), "LH")
            .unwrap();
    let directory = tempfile::Builder::new()
        .prefix("direct-nil-")
        .tempdir()
        .unwrap()
        .keep();
    let mut filament = profiles.filament("LHS PLA").unwrap();
    filament.insert("filament_flush_temp".to_owned(), json!(["nil"]));
    for (kind, mut fields) in [
        (
            "machine",
            profiles.machine("LH Stinger 0.4 nozzle").unwrap(),
        ),
        (
            "process",
            profiles.process("0.20mm Daily @LH Stinger").unwrap(),
        ),
        ("filament", filament),
    ] {
        assert!(!fields.contains_key("inherits"));
        fields.insert("type".to_owned(), json!(kind));
        fields.insert("from".to_owned(), json!("system"));
        fields.insert("instantiation".to_owned(), json!("true"));
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let input = std::fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    std::fs::write(directory.join("cube10.stl"), &input).unwrap();
    let ares = crate::runner::profile_entry::flattened::slice_flattened(
        (&input, "cube10.stl"),
        &[
            directory.join("machine.json"),
            directory.join("process.json"),
        ],
        &[directory.join("filament.json")],
        &ConfigOverrides::default(),
        Placement::None,
    )
    .unwrap_err();
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let failure = stages::run(
        runner.bin(),
        [
            "--load-settings".to_owned(),
            join_paths(
                [
                    directory.join("machine.json"),
                    directory.join("process.json"),
                ]
                .iter(),
            ),
            "--load-filaments".to_owned(),
            directory.join("filament.json").to_str().unwrap().to_owned(),
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            directory.join("native.3mf").to_str().unwrap().to_owned(),
            directory.join("cube10.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Export,
        &directory,
    )
    .unwrap_err();
    let captured = failure.captured.as_ref().unwrap();
    let expected = "filament_flush_temp: nil not in range [0, 1500]";
    write_json(
        &directory.join("result.json"),
        &json!({
            "native_code": captured.code,
            "native_stderr": String::from_utf8_lossy(&captured.stderr),
            "ares": ares.to_string(),
            "source": "OrcaSlicer.cpp:2121-2123 (typed file without inherits)",
        }),
    );
    assert_eq!(captured.code, Some(if cfg!(unix) { 238 } else { -18 }));
    assert_eq!(
        captured.stderr,
        format!("Param values in 3mf/config error: \n{expected}\n").as_bytes(),
        "evidence {directory:?}"
    );
    assert!(
        matches!(ares, ares_core::SliceError::InvalidValuesInProject(text) if text == expected),
        "evidence {directory:?}"
    );
}

async fn compare(key: &str, parent: Value, child: Value, expected: Value) {
    let root = repo_root();
    let profiles = root.join("OrcaSlicer/resources/profiles");
    let machine = "LH Stinger 0.4 nozzle";
    let process = "0.20mm Daily @LH Stinger";
    let filament = "LHS PLA";
    let mut catalog = json!({
        "vendors": [vendor(&profiles, "OrcaFilamentLibrary"), vendor(&profiles, "LH")],
        "imports": [],
    });
    let files = &mut catalog["vendors"][1]["files"];
    files["filament/fdm_filament_pla.json"][key] = parent;
    files[format!("filament/{filament}.json")][key] = child;
    let selection = json!({"machine": machine, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("vendor-nil-parent-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input =
        include_bytes!("../../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    std::fs::write(directory.join("input.3mf"), input).unwrap();
    let overrides = ConfigOverrides::default();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let ares = ares_core::profile_settings(
        input,
        ProfileSliceRequest {
            filename: "input.3mf",
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        },
    )
    .await
    .unwrap();
    let ares = serde_json::to_value(ares).unwrap();
    write_json(&directory.join("ares-settings.json"), &ares);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let presets = native_user_presets::resolved(&directory, &profiles, &catalog, &selection);
    stages::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            presets.datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            join_paths(presets.settings.iter()),
            "--load-filaments".to_owned(),
            join_paths(presets.filaments.iter()),
            "--export-settings".to_owned(),
            directory
                .join("native-settings.json")
                .to_str()
                .unwrap()
                .to_owned(),
            directory.join("input.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Export,
        &directory,
    )
    .unwrap();
    let native: Value =
        serde_json::from_slice(&std::fs::read(directory.join("native-settings.json")).unwrap())
            .unwrap();
    write_json(
        &directory.join("result.json"),
        &json!({
            "key": key,
            "native": native[key],
            "ares": ares[key],
            "source": "Preset.cpp:313-370; PresetBundle.cpp:7781-7848",
        }),
    );
    assert_eq!(
        native[key], expected,
        "invalid Native fixture: {directory:?}"
    );
    assert_eq!(ares[key], native[key], "evidence {directory:?}");
}
