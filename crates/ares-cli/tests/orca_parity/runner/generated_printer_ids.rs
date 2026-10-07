//! Public-byte generated-id witness, PrintConfig.cpp:9004–9060 and9634ff.
use super::{ExportedCase, OrcaRunner, application, repo_root};
use serde_json::{Value, json};

#[test]
fn generated_printer_ids_incomplete_map_matches_complete_native_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        std::fs::read(repo_root().join("tests/parity/generated-printer-ids/input.3mf")).unwrap();
    // Admit the defined two-nozzle selectors before any production correction.
    application::verify_value(
        &project,
        "nozzle_volume_type",
        &json!(["Standard", "Standard"]),
    )
    .unwrap();
    application::verify_value(
        &project,
        "extruder_type",
        &json!(["Direct Drive", "Bowden"]),
    )
    .unwrap();
    application::verify_value(&project, "printer_extruder_id", &json!(["1", "2"])).unwrap();
    application::verify_value(
        &project,
        "printer_extruder_variant",
        &json!([
            "Direct Drive Standard",
            "Direct Drive High Flow",
            "Bowden Standard",
            "Bowden High Flow"
        ]),
    )
    .unwrap();
    application::verify_value(
        &project,
        "extruder_variant_list",
        &json!([
            "Direct Drive Standard,Direct Drive High Flow",
            "Bowden Standard,Bowden High Flow"
        ]),
    )
    .unwrap();
    let project_path = runner.work.join("generated-printer-ids.3mf");
    let output_dir = runner.work.join("generated-printer-ids-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir(&output_dir).unwrap();
    let exported = ExportedCase {
        label: "generated-printer-ids/incomplete-map/preserved-original-input".into(),
        project,
        project_path,
        output_dir,
    };
    let case = runner.slice_case(&exported).unwrap();
    let receipt = std::fs::read_dir(&runner.work)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("cache.json"))
        .filter(|path| path.is_file())
        .map(|path| serde_json::from_slice::<Value>(&std::fs::read(path).unwrap()).unwrap())
        .find(|receipt| receipt["stage"] == "Slice")
        .unwrap();
    let outcome = crate::artifacts::compare(
        &crate::artifacts::root_from_env().unwrap(),
        &case.label,
        Ok(&case.project),
        Ok(&case.reference),
        json!({"kind":"native-cache-v2", "producer_identity":receipt["producer_digest"],
            "producer":receipt["producer"], "request_digest":receipt["request_digest"]}),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
