use serde_json::{Value, json};

use super::{compare_expected, fixture, native_environment};

fn expected(project: &Value) -> Value {
    json!({
        "filament_self_index": ["1", "2"], "filament_flow_ratio": ["0.91", "0.92"],
        "filament_extruder_variant": project["filament_extruder_variant"],
        "filament_colour": project["filament_colour"],
        "filament_settings_id": project["filament_settings_id"]
    })
}

#[tokio::test]
async fn reordered_expanded_variants_restore_nullable_values_by_name() {
    let (mut project, mut catalog) = fixture(&["1", "1", "2"]);
    project["filament_extruder_variant"] = json!([
        "Direct Drive Standard",
        "Direct Drive High Flow",
        "Direct Drive Standard"
    ]);
    project["filament_flow_ratio"] = json!(["0.91", "0.93", "0.92"]);
    project["filament_retraction_length"] = json!(["nil", "0.6", "0.7"]);
    project["different_settings_to_system"] = json!([
        "layer_height",
        "filament_flow_ratio;filament_retraction_length",
        "filament_flow_ratio;filament_retraction_length",
        ""
    ]);
    let files = &mut catalog["vendors"][0]["files"];
    files["filament/0.json"]["filament_extruder_variant"] =
        json!(["Direct Drive High Flow", "Direct Drive Standard", "Bowden"]);
    files["filament/0.json"]["filament_flow_ratio"] = json!(["1.02", "1.03", "1.05"]);
    files["filament/0.json"]["filament_retraction_length"] = json!(["nil", "nil", "nil"]);
    files["filament/1.json"]["filament_retraction_length"] = json!(["nil"]);
    let mut result = expected(&project);
    result["filament_self_index"] = json!(["1", "1", "1", "2"]);
    result["filament_extruder_variant"] = json!([
        "Direct Drive High Flow",
        "Direct Drive Standard",
        "Bowden",
        "Direct Drive Standard"
    ]);
    result["filament_flow_ratio"] = json!(["0.93", "0.91", "1.05", "0.92"]);
    result["filament_retraction_length"] = json!(["0.6", "nil", "nil", "0.7"]);
    compare_expected(project, catalog, result, false).await;
}

const DEVICE: &str = "filament_dev_ams_drying_temperature";

fn device_fixture() -> (Value, Value) {
    let (project, mut catalog) = fixture(&["1", "2"]);
    let files = &mut catalog["vendors"][0]["files"];
    files["filament/0.json"][DEVICE] = json!(["41", "42"]);
    files["filament/1.json"][DEVICE] = json!(["51"]);
    (project, catalog)
}

#[tokio::test]
async fn device_vectors_append_every_element_in_slot_order() {
    let (project, catalog) = device_fixture();
    let mut result = expected(&project);
    result[DEVICE] = json!(["41", "42", "51"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn empty_device_json_float_vector_deserializes_zero() {
    let (project, mut catalog) = device_fixture();
    catalog["vendors"][0]["files"]["filament/0.json"][DEVICE] = json!([]);
    let mut result = expected(&project);
    result[DEVICE] = json!(["0", "51"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn empty_non_device_float_json_array_deserializes_zero() {
    empty_density(json!([])).await;
}

#[tokio::test]
async fn bare_empty_non_device_float_json_string_deserializes_zero() {
    empty_density(json!("")).await;
}

async fn empty_density(value: Value) {
    let (project, mut catalog) = fixture(&["1", "2"]);
    let files = &mut catalog["vendors"][0]["files"];
    files["filament/0.json"]["filament_density"] = value;
    files["filament/1.json"]["filament_density"] = json!(["1.24"]);
    let mut result = expected(&project);
    result["filament_density"] = json!(["0", "1.24"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn empty_nullable_float_json_array_deserializes_zero_without_losing_other_nil() {
    let (project, mut catalog) = fixture(&["1", "2"]);
    catalog["vendors"][0]["files"]["filament/0.json"]["filament_retraction_length"] = json!([]);
    let mut result = expected(&project);
    result["filament_retraction_length"] = json!(["0", "nil"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn nonempty_non_device_float_json_array_keeps_values() {
    let (project, mut catalog) = fixture(&["1", "2"]);
    let files = &mut catalog["vendors"][0]["files"];
    files["filament/0.json"]["filament_density"] = json!(["1.1"]);
    files["filament/1.json"]["filament_density"] = json!(["1.24"]);
    let mut result = expected(&project);
    result["filament_density"] = json!(["1.1", "1.24"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn bbl_machine_inherited_empty_float_array_matches_native() {
    let Some(path) = native_environment::value("ARES_PARITY_PROJECT_PRESET_CATALOG") else {
        assert!(native_environment::value("ARES_REQUIRE_ORCA").is_none());
        return;
    };
    let source: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let vendor = source["vendors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vendor| vendor["id"] == "BBL")
        .unwrap()
        .clone();
    assert_eq!(
        vendor["files"]["machine/fdm_bbl_3dp_001_common.json"]["extruder_printable_height"],
        json!([])
    );
    let (mut project, mut catalog) = fixture(&["1", "2"]);
    project["printer_settings_id"] = json!("Bambu Lab A1 0.4 nozzle");
    catalog["vendors"].as_array_mut().unwrap().push(vendor);
    let mut result = expected(&project);
    result["extruder_printable_height"] = json!(["0"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn empty_string_vector_array_contributes_no_float_zero() {
    empty_string_vector(json!([])).await;
}

#[tokio::test]
async fn bare_empty_string_vector_contributes_no_float_zero() {
    empty_string_vector(json!("")).await;
}

async fn empty_string_vector(value: Value) {
    let (project, mut catalog) = fixture(&["1", "2"]);
    let key = "filament_dev_ams_drying_ams_limitations";
    let files = &mut catalog["vendors"][0]["files"];
    files["filament/0.json"][key] = value;
    files["filament/1.json"][key] = json!(["SECOND"]);
    let mut result = expected(&project);
    result[key] = json!(["SECOND"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn sparse_device_vector_uses_registered_slot_default() {
    let (project, mut catalog) = device_fixture();
    catalog["vendors"][0]["files"]["filament/0.json"]
        .as_object_mut()
        .unwrap()
        .remove(DEVICE);
    let mut result = expected(&project);
    result[DEVICE] = json!(["0", "51"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn absent_device_vectors_materialize_both_registered_defaults() {
    let (project, catalog) = fixture(&["1", "2"]);
    let mut result = expected(&project);
    result[DEVICE] = json!(["0", "0"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn empty_differences_keep_existing_device_vector() {
    let (mut project, catalog) = device_fixture();
    project["different_settings_to_system"] = json!(["", "", "", ""]);
    project[DEVICE] = json!(["71", "72"]);
    let mut result = expected(&project);
    result[DEVICE] = json!(["71", "72"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn process_only_difference_triggers_filament_and_device_refresh() {
    let (mut project, catalog) = device_fixture();
    project["different_settings_to_system"] = json!(["layer_height", "", "", ""]);
    let mut result = expected(&project);
    result["filament_flow_ratio"] = json!(["1.02", "1.04"]);
    result[DEVICE] = json!(["41", "42", "51"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn partial_system_lookup_keeps_existing_device_vector_during_refresh() {
    let (mut project, catalog) = device_fixture();
    project["filament_settings_id"][1] = json!("Unlisted Synthetic Material");
    project[DEVICE] = json!(["71", "72"]);
    let mut result = expected(&project);
    result[DEVICE] = json!(["71", "72"]);
    compare_expected(project, catalog, result, false).await;
}

#[tokio::test]
async fn missing_device_vector_fills_optional_miss_with_registered_default() {
    let (mut project, catalog) = device_fixture();
    project["filament_settings_id"][1] = json!("Unlisted Synthetic Material");
    project["different_settings_to_system"] = json!(["", "", "", ""]);
    let mut result = expected(&project);
    result[DEVICE] = json!(["41", "42", "0"]);
    compare_expected(project, catalog, result, false).await;
}
