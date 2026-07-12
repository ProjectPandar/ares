use super::super::{
    extruder_option_keys, extruder_retract_keys, filament_option_keys,
    filament_options_with_variant, filament_retract_keys, option_definition,
    print_options_with_variant, printer_extruder_options, printer_options_with_variant_1,
    printer_options_with_variant_2,
};

const EXTRUDER_OPTION_KEYS: &[&str] = &[
    "extruder_type",
    "nozzle_diameter",
    "default_nozzle_volume_type",
    "min_layer_height",
    "max_layer_height",
    "extruder_offset",
    "extruder_printable_height",
    "nozzle_volume",
    "nozzle_type",
    "nozzle_flush_dataset",
    "retraction_length",
    "z_hop",
    "z_hop_types",
    "travel_slope",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retraction_speed",
    "deretraction_speed",
    "retract_before_wipe",
    "retract_restart_extra",
    "retraction_minimum_travel",
    "wipe",
    "wipe_distance",
    "retract_when_changing_layer",
    "retract_length_toolchange",
    "retract_restart_extra_toolchange",
    "extruder_colour",
    "default_filament_profile",
    "retraction_distances_when_cut",
    "long_retractions_when_cut",
];

const EXTRUDER_RETRACT_KEYS: &[&str] = &[
    "deretraction_speed",
    "long_retractions_when_cut",
    "retract_before_wipe",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retract_restart_extra",
    "retract_when_changing_layer",
    "retraction_distances_when_cut",
    "retraction_length",
    "retraction_minimum_travel",
    "retraction_speed",
    "travel_slope",
    "wipe",
    "wipe_distance",
    "z_hop",
    "z_hop_types",
];

const FILAMENT_OPTION_KEYS: &[&str] = &[
    "filament_diameter",
    "min_layer_height",
    "max_layer_height",
    "volumetric_speed_coefficients",
    "retraction_length",
    "z_hop",
    "z_hop_types",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retraction_speed",
    "deretraction_speed",
    "retract_before_wipe",
    "retract_restart_extra",
    "retraction_minimum_travel",
    "wipe",
    "wipe_distance",
    "retract_when_changing_layer",
    "retract_length_toolchange",
    "retract_restart_extra_toolchange",
    "filament_colour",
    "default_filament_profile",
    "retraction_distances_when_cut",
    "long_retractions_when_cut",
];

const FILAMENT_RETRACT_KEYS: &[&str] = &[
    "deretraction_speed",
    "long_retractions_when_cut",
    "retract_before_wipe",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retract_restart_extra",
    "retract_when_changing_layer",
    "retraction_distances_when_cut",
    "retraction_length",
    "retraction_minimum_travel",
    "retraction_speed",
    "wipe",
    "wipe_distance",
    "z_hop",
    "z_hop_types",
];

const PRINT_OPTIONS_WITH_VARIANT_KEYS: &[&str] = &["print_extruder_id", "print_extruder_variant"];

const FILAMENT_OPTIONS_WITH_VARIANT_KEYS: &[&str] = &[
    "activate_air_filtration",
    "activate_air_filtration_during_print",
    "activate_air_filtration_on_completion",
    "complete_print_exhaust_fan_speed",
    "during_print_exhaust_fan_speed",
    "filament_adaptive_volumetric_speed",
    "filament_cooling_before_tower",
    "filament_deretraction_speed",
    "filament_extruder_variant",
    "filament_flow_ratio",
    "filament_flush_temp",
    "filament_flush_volumetric_speed",
    "filament_ironing_flow",
    "filament_ironing_inset",
    "filament_ironing_spacing",
    "filament_ironing_speed",
    "filament_long_retractions_when_cut",
    "filament_max_volumetric_speed",
    "filament_retract_before_wipe",
    "filament_retract_lift_above",
    "filament_retract_lift_below",
    "filament_retract_lift_enforce",
    "filament_retract_restart_extra",
    "filament_retract_when_changing_layer",
    "filament_retraction_distances_when_cut",
    "filament_retraction_length",
    "filament_retraction_minimum_travel",
    "filament_retraction_speed",
    "filament_wipe",
    "filament_wipe_distance",
    "filament_z_hop",
    "filament_z_hop_types",
    "long_retractions_when_ec",
    "nozzle_temperature",
    "nozzle_temperature_initial_layer",
    "retraction_distances_when_ec",
    "volumetric_speed_coefficients",
];

const PRINTER_EXTRUDER_OPTION_KEYS: &[&str] = &[
    "default_nozzle_volume_type",
    "extruder_printable_area",
    "extruder_printable_height",
    "extruder_type",
    "max_layer_height",
    "min_layer_height",
    "nozzle_diameter",
];

const PRINTER_OPTIONS_WITH_VARIANT_1_KEYS: &[&str] = &[
    "deretraction_speed",
    "long_retractions_when_cut",
    "nozzle_flush_dataset",
    "nozzle_type",
    "nozzle_volume",
    "printer_extruder_id",
    "printer_extruder_variant",
    "retract_before_wipe",
    "retract_length_toolchange",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retract_restart_extra",
    "retract_restart_extra_toolchange",
    "retract_when_changing_layer",
    "retraction_distances_when_cut",
    "retraction_length",
    "retraction_minimum_travel",
    "retraction_speed",
    "travel_slope",
    "wipe",
    "wipe_distance",
    "z_hop",
    "z_hop_types",
];

const PRINTER_OPTIONS_WITH_VARIANT_2_KEYS: &[&str] = &[
    "machine_max_acceleration_e",
    "machine_max_acceleration_extruding",
    "machine_max_acceleration_retracting",
    "machine_max_acceleration_travel",
    "machine_max_acceleration_x",
    "machine_max_acceleration_y",
    "machine_max_acceleration_z",
    "machine_max_jerk_e",
    "machine_max_jerk_x",
    "machine_max_jerk_y",
    "machine_max_jerk_z",
    "machine_max_speed_e",
    "machine_max_speed_x",
    "machine_max_speed_y",
    "machine_max_speed_z",
];

#[test]
fn extruder_and_filament_key_lists_match_upstream_print_config_order() {
    assert_eq!(extruder_option_keys(), EXTRUDER_OPTION_KEYS);
    assert_eq!(extruder_retract_keys(), EXTRUDER_RETRACT_KEYS);
    assert_eq!(filament_option_keys(), FILAMENT_OPTION_KEYS);
    assert_eq!(filament_retract_keys(), FILAMENT_RETRACT_KEYS);
}

#[test]
fn retract_key_lists_preserve_upstream_sorted_order() {
    for keys in [extruder_retract_keys(), filament_retract_keys()] {
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    }
}

#[test]
fn extruder_and_filament_key_lists_are_covered_by_option_definitions() {
    for key in extruder_option_keys()
        .iter()
        .chain(extruder_retract_keys())
        .chain(filament_option_keys())
        .chain(filament_retract_keys())
    {
        assert!(
            option_definition(key).is_some(),
            "missing definition for {key}"
        );
    }
}

#[test]
fn variant_key_lists_match_upstream_print_config_sets() {
    assert_eq!(
        print_options_with_variant(),
        PRINT_OPTIONS_WITH_VARIANT_KEYS
    );
    assert_eq!(
        filament_options_with_variant(),
        FILAMENT_OPTIONS_WITH_VARIANT_KEYS
    );
    assert_eq!(printer_extruder_options(), PRINTER_EXTRUDER_OPTION_KEYS);
    assert_eq!(
        printer_options_with_variant_1(),
        PRINTER_OPTIONS_WITH_VARIANT_1_KEYS
    );
    assert_eq!(
        printer_options_with_variant_2(),
        PRINTER_OPTIONS_WITH_VARIANT_2_KEYS
    );
}

#[test]
fn variant_key_lists_are_sorted_and_unique() {
    for keys in [
        print_options_with_variant(),
        filament_options_with_variant(),
        printer_extruder_options(),
        printer_options_with_variant_1(),
        printer_options_with_variant_2(),
    ] {
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    }
}

#[test]
fn variant_key_lists_are_covered_by_option_definitions() {
    for key in print_options_with_variant()
        .iter()
        .chain(filament_options_with_variant())
        .chain(printer_extruder_options())
        .chain(printer_options_with_variant_1())
        .chain(printer_options_with_variant_2())
    {
        assert!(
            option_definition(key).is_some(),
            "missing definition for {key}"
        );
    }
}
