use crate::runner::{OrcaRunner, native_api_cli, profile_entry::write_json, repo_root, stages};
use serde_json::{Value, json};
use std::fs;

#[test]
fn singleton_toolchange_retract_length_expands_to_native_variants() {
    // PrintConfig.cpp:10171 owns this nullable vector by filament variant.
    check(
        "retract-length-toolchange",
        "filament_retract_length_toolchange",
    );
}

#[test]
fn singleton_toolchange_restart_extra_expands_to_native_variants() {
    // PrintConfig.cpp:10172 owns this nullable vector by filament variant.
    check(
        "retract-restart-extra-toolchange",
        "filament_retract_restart_extra_toolchange",
    );
}

#[test]
fn singleton_minimal_purge_expands_to_native_variants() {
    // PrintConfig.cpp:10210 owns this vector by filament variant.
    check(
        "minimal-purge-on-wipe-tower",
        "filament_minimal_purge_on_wipe_tower",
    );
}

fn check(name: &str, key: &str) {
    let retained = repo_root().join("tests/parity/filament-variant-owning-members");
    let provenance: Value =
        serde_json::from_slice(&fs::read(retained.join("provenance.json")).unwrap()).unwrap();
    let record = provenance["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == name)
        .unwrap();
    assert_eq!(record["key"], key);
    let project = fs::read(retained.join(record["file"].as_str().unwrap())).unwrap();
    assert_eq!(native_api_cli::sha(&project), record["input_sha256"]);
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let native =
        native_api_cli::NativeCase::slice(&runner, &project, &format!("filament-variant-{name}-"));
    let exported = native.root.join("native-normalized.3mf");
    stages::run(
        runner.bin(),
        [
            "--arrange".to_owned(),
            "0".to_owned(),
            "--export-3mf".to_owned(),
            exported.to_str().unwrap().to_owned(),
            native.root.join("input.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Export,
        &native.root,
    )
    .unwrap();
    let exported_bytes = fs::read(&exported).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&exported_bytes)).unwrap();
    let normalized: Value =
        serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap())
            .unwrap();
    assert_eq!(normalized[key], record["expected_native_variant_values"]);
    assert_eq!(normalized["filament_self_index"], json!(["1", "1", "1"]));
    assert_eq!(
        normalized["filament_extruder_variant"],
        provenance["filament_extruder_variant"]
    );
    write_json(
        &native.root.join("native-variant-premise.json"),
        &json!({
            "key": key,
            "source_citation": record["source_citation"],
            "input_sha256": native_api_cli::sha(&project),
            "export_sha256": native_api_cli::sha(&exported_bytes),
            "native_values": normalized[key],
            "native_premises": "PASS",
        }),
    );

    let loaded = ares_core::load_project(&project);
    let actual = loaded
        .as_ref()
        .map(|loaded| serde_json::to_value(&loaded.settings().filament).unwrap()[key].clone())
        .map_err(ToString::to_string);
    write_json(
        &native.root.join("public-variant-import.json"),
        &json!({
            "key": key,
            "source_citation": record["source_citation"],
            "input_sha256": native_api_cli::sha(&project),
            "native_values": normalized[key],
            "public_api": "load_project / Project.settings().filament",
            "api_values": actual,
            "matches": actual.as_ref().is_ok_and(|values| values == &normalized[key]),
        }),
    );
    assert_eq!(actual.unwrap(), normalized[key], "variant ownership: {key}");
    native.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}
