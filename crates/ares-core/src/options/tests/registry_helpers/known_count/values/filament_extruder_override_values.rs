use serde_json::{Map, Value, json};

pub(super) fn insert_known_filament_extruder_override_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("filament_deretraction_speed", json!(["nil"])),
        ("filament_long_retractions_when_cut", json!(["nil"])),
        ("filament_retract_before_wipe", json!(["nil"])),
        ("filament_retract_lift_above", json!(["nil"])),
        ("filament_retract_lift_below", json!(["nil"])),
        ("filament_retract_lift_enforce", json!(["nil"])),
        ("filament_retract_restart_extra", json!(["nil"])),
        ("filament_retract_when_changing_layer", json!(["nil"])),
        ("filament_retraction_distances_when_cut", json!(["nil"])),
        ("filament_retraction_length", json!(["nil"])),
        ("filament_retraction_minimum_travel", json!(["nil"])),
        ("filament_retraction_speed", json!(["nil"])),
        ("filament_wipe", json!(["nil"])),
        ("filament_wipe_distance", json!(["nil"])),
        ("filament_z_hop", json!(["nil"])),
        ("filament_z_hop_types", json!(["nil"])),
    ] {
        values.insert(key.to_owned(), value);
    }
}
