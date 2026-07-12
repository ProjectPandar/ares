use serde_json::{Map, Value, json};

pub(super) fn insert_known_extruder_variant_id_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("extruder_ams_count", json!([])),
        ("extruder_variant_list", json!(["Direct Drive Standard"])),
        (
            "filament_extruder_variant",
            json!(["Direct Drive Standard"]),
        ),
        ("filament_self_index", json!([1])),
        ("master_extruder_id", json!(1)),
        ("print_extruder_id", json!([1])),
        ("print_extruder_variant", json!(["Direct Drive Standard"])),
        ("printer_extruder_id", json!([1])),
        ("printer_extruder_variant", json!(["Direct Drive Standard"])),
    ] {
        values.insert(key.to_owned(), value);
    }
}
