use super::super::*;
use serde_json::{Map, Value, json};

const OBSOLETE_KEYS: &[&str] = &[
    "acceleration",
    "scale",
    "rotate",
    "duplicate",
    "duplicate_grid",
    "bed_size",
    "print_center",
    "g0",
    "wipe_tower_per_color_wipe",
    "support_sharp_tails",
    "support_remove_small_overhangs",
    "support_with_sheath",
    "tree_support_collision_resolution",
    "tree_support_with_infill",
    "max_volumetric_speed",
    "max_print_speed",
    "support_closing_radius",
    "remove_freq_sweep",
    "remove_bed_leveling",
    "remove_extrusion_calibration",
    "support_transition_line_width",
    "support_transition_speed",
    "bed_temperature",
    "bed_temperature_initial_layer",
    "can_switch_nozzle_type",
    "can_add_auxiliary_fan",
    "extra_flush_volume",
    "spaghetti_detector",
    "adaptive_layer_height",
    "z_hop_type",
    "z_lift_type",
    "bed_temperature_difference",
    "long_retraction_when_cut",
    "retraction_distance_when_cut",
    "internal_bridge_support_thickness",
    "top_area_threshold",
    "reduce_wall_solid_infill",
    "filament_load_time",
    "filament_unload_time",
    "smooth_coefficient",
    "overhang_totally_speed",
    "overhang_speed_classic",
    "filament_prime_volume",
];

#[test]
fn drops_all_legacy_obsolete_keys() {
    let mut values = Map::new();
    for key in OBSOLETE_KEYS {
        values.insert((*key).to_owned(), json!("legacy"));
    }
    values.insert("future_orca_key".to_owned(), json!("preserved"));

    let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

    for key in OBSOLETE_KEYS {
        assert!(
            !options.values().contains_key(*key),
            "{key} was not dropped"
        );
    }
    assert_eq!(options.values()["future_orca_key"], json!("preserved"));
}

#[test]
fn drops_obsolete_keys_for_any_json_value_shape() {
    for value in [
        json!("legacy"),
        json!(42),
        json!(true),
        json!(["legacy"]),
        json!({"legacy": true}),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "acceleration": value
        }))
        .unwrap();

        assert!(!options.values().contains_key("acceleration"));
    }
}

#[test]
fn silent_mode_survives_legacy_normalization_for_runtime_consumption() {
    let options: SliceOptions = serde_json::from_value(json!({
        "silent_mode": true,
        "bed_temperature": "60"
    }))
    .unwrap();

    assert_eq!(options.values()["silent_mode"], json!(true));
    assert!(!options.values().contains_key("bed_temperature"));
}

#[test]
fn keeps_non_obsolete_unknown_keys_until_final_validation_is_ported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "future_orca_key": {"nested": true},
        "bed_temperature": "60"
    }))
    .unwrap();

    assert_eq!(options.values()["future_orca_key"], json!({"nested": true}));
    assert!(!options.values().contains_key("bed_temperature"));
}

#[test]
fn obsolete_ignore_coexists_with_prior_legacy_migrations() {
    let options: SliceOptions = serde_json::from_value(json!({
        "enable_wipe_tower": true,
        "wall_direction": "auto",
        "bed_temperature": "60"
    }))
    .unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(options.values()["wall_direction"], json!("ccw"));
    assert!(!options.values().contains_key("enable_wipe_tower"));
    assert!(!options.values().contains_key("bed_temperature"));
}
