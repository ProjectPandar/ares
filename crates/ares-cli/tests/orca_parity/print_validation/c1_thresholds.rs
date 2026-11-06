//! PrintConfig.cpp:10340-10457: independent public rejection and complete-byte gates.
use serde_json::json;

#[path = "c1_thresholds/run.rs"]
mod run;
use run::verify;

macro_rules! threshold_case {
    ($name:ident, $key:literal, $value:literal, $expected:expr $(, $dirty:literal)?) => {
        #[tokio::test]
        async fn $name() {
            let value = if $key.starts_with("filament_") {
                json!([$value])
            } else {
                json!($value)
            };
            verify(json!({$key: value}), $expected, &[$($dirty)?]).await;
        }
    };
}

threshold_case!(
    c1_layer_height_zero,
    "layer_height",
    "0",
    Some("layer_height: invalid value 0.000000")
);
threshold_case!(
    c1_layer_height_positive,
    "layer_height",
    "0.001",
    None,
    "; different_settings_to_system = layer_height;;"
);
threshold_case!(
    c1_initial_height_zero,
    "initial_layer_print_height",
    "0",
    Some("initial_layer_print_height: invalid value 0.000000")
);
threshold_case!(
    c1_initial_height_positive,
    "initial_layer_print_height",
    "0.001",
    None,
    "; different_settings_to_system = initial_layer_print_height;;"
);
threshold_case!(
    c1_bridge_flow_zero,
    "bridge_flow",
    "0",
    Some("bridge_flow: invalid value 0.000000")
);
threshold_case!(
    c1_bridge_flow_positive,
    "bridge_flow",
    "0.001",
    None,
    "; different_settings_to_system = bridge_flow;;"
);
threshold_case!(
    c1_internal_bridge_flow_zero,
    "internal_bridge_flow",
    "0",
    Some("internal_bridge_flow: invalid value 0.000000")
);
threshold_case!(
    c1_internal_bridge_flow_positive,
    "internal_bridge_flow",
    "0.001",
    None,
    "; different_settings_to_system = internal_bridge_flow;;"
);
threshold_case!(
    c1_clearance_radius_zero,
    "extruder_clearance_radius",
    "0",
    Some("extruder_clearance_radius: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_radius_positive,
    "extruder_clearance_radius",
    "0.001",
    None,
    "; different_settings_to_system = ;;extruder_clearance_radius"
);
threshold_case!(
    c1_clearance_rod_zero,
    "extruder_clearance_height_to_rod",
    "0",
    Some("extruder_clearance_height_to_rod: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_rod_positive,
    "extruder_clearance_height_to_rod",
    "0.001",
    None,
    "; different_settings_to_system = ;;extruder_clearance_height_to_rod"
);
threshold_case!(
    c1_clearance_lid_zero,
    "extruder_clearance_height_to_lid",
    "0",
    Some("extruder_clearance_height_to_lid: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_lid_positive,
    "extruder_clearance_height_to_lid",
    "0.001",
    None,
    "; different_settings_to_system = ;;extruder_clearance_height_to_lid"
);
threshold_case!(
    c1_nozzle_height_zero,
    "nozzle_height",
    "0",
    Some("nozzle_height: invalid value 0.000000")
);
threshold_case!(
    c1_nozzle_height_positive,
    "nozzle_height",
    "0.001",
    None,
    "; different_settings_to_system = ;;nozzle_height"
);
threshold_case!(
    c1_filament_flow_zero,
    "filament_flow_ratio",
    "0",
    Some("filament_flow_ratio: invalid value 0")
);
threshold_case!(
    c1_filament_flow_positive,
    "filament_flow_ratio",
    "0.001",
    None,
    "; different_settings_to_system = ;filament_flow_ratio;"
);
threshold_case!(
    c1_filament_diameter_schema_zero,
    "filament_diameter",
    "0",
    Some("filament_diameter: invalid value 0")
);
threshold_case!(
    c1_filament_diameter_below,
    "filament_diameter",
    "0.999",
    Some("filament_diameter: invalid value 0.999")
);
threshold_case!(
    c1_filament_diameter_threshold,
    "filament_diameter",
    "1",
    None,
    "; different_settings_to_system = ;filament_diameter;"
);
threshold_case!(
    c1_filament_diameter_above,
    "filament_diameter",
    "1.001",
    None,
    "; different_settings_to_system = ;filament_diameter;"
);

threshold_case!(
    c1_override_scalar_same_value,
    "layer_height",
    "0.200",
    None,
    "; different_settings_to_system = ;;"
);
#[tokio::test]
async fn c1_override_vector_front_fallback_equivalent() {
    verify(
        json!({"nozzle_temperature": ["190", "190", "190"]}),
        None,
        &["; different_settings_to_system = ;;"],
    )
    .await;
}

#[tokio::test]
async fn c1_unchanged_control() {
    verify(json!({}), None, &[]).await;
}
