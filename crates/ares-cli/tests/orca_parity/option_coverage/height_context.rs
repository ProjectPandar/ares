//! Print.cpp:1639-1694: finite one-nozzle height scope, with width isolation.
use serde_json::{Map, Value, json};

pub(super) fn activate(
    key: &str,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
) {
    if !matches!(key, "layer_height" | "initial_layer_print_height") {
        return;
    }
    machine.insert("nozzle_diameter".into(), json!(["0.4"]));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    for (field, value) in dependencies() {
        process.insert(field, value);
    }
}

fn dependencies() -> Map<String, Value> {
    let mut fields = Map::new();
    for field in ["layer_height", "initial_layer_print_height"] {
        fields.insert(field.into(), json!("0.2"));
    }
    for field in [
        "line_width",
        "initial_layer_line_width",
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
    ] {
        fields.insert(field.into(), json!("0.6"));
    }
    for (field, value) in [
        ("bridge_line_width", "0.4"),
        ("thick_bridges", "1"),
        ("thick_internal_bridges", "1"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
    ] {
        fields.insert(field.into(), json!(value));
    }
    fields
}

pub(super) fn metadata() -> Value {
    json!({
        "scope":"five finite heights, single 0.4mm nozzle; not all physical heights",
        "source":"Print.cpp:1639-1694; PrintConfig.cpp:10337-10351; Slicing.cpp:69-146,809-868",
        "nozzle_diameter":["0.4"], "use_relative_e_distances":"1",
        "process_dependencies":dependencies(),
        "override":"selected height replaces only its own 0.2mm dependency",
        "deferred":["mixed nozzles", "raft/support", "object/region ranges", "adaptive heights",
            "minimum positive float", "arbitrary Flow feasibility"],
    })
}

pub(super) fn verify(
    project: &[u8],
    key: &str,
    requested: &Value,
) -> Result<Value, crate::runner::stages::StageError> {
    let mut fields = dependencies();
    fields.insert(key.into(), requested.clone());
    fields.insert("nozzle_diameter".into(), json!(["0.4"]));
    fields.insert("use_relative_e_distances".into(), json!("1"));
    let mut proofs = Map::new();
    for (field, value) in fields {
        proofs.insert(
            field.clone(),
            crate::runner::application::verify_value(project, &field, &value)?,
        );
    }
    Ok(json!({"domain":metadata(),"requested":requested,"application":proofs}))
}
