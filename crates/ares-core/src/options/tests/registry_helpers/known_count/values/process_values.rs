use serde_json::{Map, Value, json};

pub(super) fn insert_known_process_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("detect_narrow_internal_solid_infill", json!(true)),
        ("detect_overhang_wall", json!(true)),
        ("different_settings_to_system", json!([])),
    ] {
        values.insert(key.to_owned(), value);
    }
}
