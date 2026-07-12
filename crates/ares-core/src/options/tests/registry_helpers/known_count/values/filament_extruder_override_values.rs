use serde_json::{Map, Value, json};

pub(super) fn insert_known_filament_extruder_override_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("filament_deretraction_speed", json!([0])),
        ("filament_long_retractions_when_cut", json!([false])),
        ("filament_retract_before_wipe", json!([100])),
        ("filament_retract_lift_above", json!([0])),
        ("filament_retract_lift_below", json!([0])),
        ("filament_retract_lift_enforce", json!(["All Surfaces"])),
        ("filament_retract_restart_extra", json!([0])),
        ("filament_retract_when_changing_layer", json!([false])),
        ("filament_retraction_distances_when_cut", json!([18])),
        ("filament_retraction_length", json!([0.8])),
        ("filament_retraction_minimum_travel", json!([2])),
        ("filament_retraction_speed", json!([30])),
        ("filament_wipe", json!([false])),
        ("filament_wipe_distance", json!([1])),
        ("filament_z_hop", json!([0.4])),
        ("filament_z_hop_types", json!(["Slope Lift"])),
    ] {
        values.insert(key.to_owned(), value);
    }
}
