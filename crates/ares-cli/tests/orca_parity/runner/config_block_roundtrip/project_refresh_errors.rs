use super::super::repo_root;
use serde_json::json;

#[test]
fn project_refresh_requires_a_supplied_named_preset() {
    let error = request(json!({"from_project": true}));
    assert!(
        error.contains("Bambu Lab A1 0.4 nozzle"),
        "missing-preset error must identify the requested machine: {error}"
    );
}

#[test]
fn project_refresh_and_explicit_selection_are_exclusive() {
    let error = request(json!({
        "from_project": true, "machine": "machine", "process": "process", "filaments": ["filament"]
    }));
    assert!(error.contains("exclusive"), "{error}");
}

#[test]
fn project_refresh_false_is_not_an_explicit_selection() {
    let error = request(json!({"from_project": false}));
    assert!(error.contains("from_project must be true"), "{error}");
}

fn request(selection: serde_json::Value) -> String {
    let input = std::fs::read(
        repo_root().join("tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"),
    )
    .unwrap();
    let selection = selection.to_string();
    let overrides = ares_core::ConfigOverrides::default();
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::profile_settings(
            input,
            ares_core::ProfileSliceRequest {
                filename: "project.3mf",
                catalog_json: "{\"vendors\":[],\"imports\":[]}",
                selection_json: &selection,
                plate: Some(1),
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
        )) {
        Ok(_) => panic!("an invalid project-refresh request must fail"),
        Err(error) => error.to_string(),
    }
}
