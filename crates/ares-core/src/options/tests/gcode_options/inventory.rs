use std::collections::{BTreeMap, BTreeSet};

use super::super::super::{
    FilamentGCodeSourceOptions, GCodeOptionSource, GCodeOptions, PrinterGCodeSourceOptions,
    ProcessGCodeSourceOptions, ProjectGCodeSourceOptions,
};
use super::{gcode_rows, inventory};

const EXPECTED_HPP_ORDER: [&str; 149] = [
    "before_layer_change_gcode",
    "printing_by_object_gcode",
    "deretraction_speed",
    "enable_arc_fitting",
    "machine_end_gcode",
    "filament_end_gcode",
    "filament_flow_ratio",
    "enable_pressure_advance",
    "pressure_advance",
    "adaptive_pressure_advance",
    "adaptive_pressure_advance_overhangs",
    "adaptive_pressure_advance_model",
    "adaptive_pressure_advance_bridges",
    "fan_kickstart",
    "fan_speedup_overhangs",
    "fan_speedup_time",
    "part_cooling_fan_min_pwm",
    "filament_diameter",
    "filament_adaptive_volumetric_speed",
    "volumetric_speed_coefficients",
    "filament_adhesiveness_category",
    "filament_density",
    "filament_type",
    "filament_soluble",
    "filament_ids",
    "filament_colour",
    "filament_vendor",
    "filament_is_support",
    "filament_printable",
    "filament_change_length",
    "filament_cost",
    "default_filament_colour",
    "temperature_vitrification",
    "filament_max_volumetric_speed",
    "required_nozzle_HRC",
    "filament_map_mode",
    "filament_map",
    "filament_extruder_variant",
    "support_object_skip_flush",
    "bed_temperature_formula",
    "physical_extruder_map",
    "nozzle_flush_dataset",
    "filament_flush_volumetric_speed",
    "filament_flush_temp",
    "scan_first_layer",
    "enable_power_loss_recovery",
    "enable_wrapping_detection",
    "wrapping_detection_layers",
    "wrapping_exclude_area",
    "gcode_add_line_number",
    "gcode_flavor",
    "time_cost",
    "layer_change_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
    "max_volumetric_extrusion_rate_slope",
    "max_volumetric_extrusion_rate_slope_segment_length",
    "extrusion_rate_smoothing_external_perimeter_only",
    "retract_before_wipe",
    "retraction_length",
    "retract_length_toolchange",
    "enable_long_retraction_when_cut",
    "retraction_distances_when_cut",
    "long_retractions_when_cut",
    "retraction_distances_when_ec",
    "long_retractions_when_ec",
    "z_hop",
    "z_hop_types",
    "travel_slope",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retract_restart_extra",
    "retract_restart_extra_toolchange",
    "retraction_speed",
    "file_start_gcode",
    "machine_start_gcode",
    "filament_start_gcode",
    "single_extruder_multi_material",
    "manual_filament_change",
    "single_extruder_multi_material_priming",
    "wipe_tower_no_sparse_layers",
    "change_filament_gcode",
    "change_extrusion_role_gcode",
    "process_change_extrusion_role_gcode",
    "filament_change_extrusion_role_gcode",
    "travel_speed",
    "travel_speed_z",
    "silent_mode",
    "machine_pause_gcode",
    "template_custom_gcode",
    "nozzle_type",
    "nozzle_hrc",
    "auxiliary_fan",
    "support_air_filtration",
    "printer_structure",
    "support_chamber_temp_control",
    "extruder_type",
    "nozzle_volume_type",
    "extruder_ams_count",
    "printer_extruder_id",
    "master_extruder_id",
    "printer_extruder_variant",
    "use_firmware_retraction",
    "use_relative_e_distances",
    "accel_to_decel_enable",
    "accel_to_decel_factor",
    "initial_layer_travel_speed",
    "initial_layer_travel_acceleration",
    "initial_layer_travel_jerk",
    "bbl_calib_mark_logo",
    "disable_m73",
    "cooling_tube_retraction",
    "cooling_tube_length",
    "high_current_on_filament_swap",
    "parking_pos_retraction",
    "extra_loading_move",
    "machine_load_filament_time",
    "machine_tool_change_time",
    "machine_unload_filament_time",
    "filament_loading_speed",
    "filament_loading_speed_start",
    "filament_unloading_speed",
    "filament_unloading_speed_start",
    "filament_toolchange_delay",
    "filament_cooling_moves",
    "filament_cooling_initial_speed",
    "filament_minimal_purge_on_wipe_tower",
    "filament_cooling_before_tower",
    "filament_tower_interface_pre_extrusion_dist",
    "filament_tower_interface_pre_extrusion_length",
    "filament_tower_ironing_area",
    "filament_tower_interface_purge_volume",
    "filament_tower_interface_print_temp",
    "filament_cooling_final_speed",
    "filament_ramming_parameters",
    "filament_multitool_ramming",
    "filament_multitool_ramming_volume",
    "filament_multitool_ramming_flow",
    "filament_stamping_loading_speed",
    "filament_stamping_distance",
    "wipe_tower_type",
    "purge_in_prime_tower",
    "enable_filament_ramming",
    "tool_change_on_wipe_tower",
    "support_multi_bed_types",
    "use_3mf",
    "small_area_infill_flow_compensation_model",
    "has_scarf_joint_seam",
];

#[test]
fn gcode_options_inventory_is_the_exact_registered_partition() {
    let inventory = inventory();
    let rows = gcode_rows(&inventory);
    assert_eq!(rows.len(), 149);
    assert_eq!(
        rows.iter()
            .map(|row| row.key.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        149
    );
    assert!(
        rows.iter()
            .all(|row| row.static_owner == "g_code_config")
    );

    let expected_sources = [
        (
            "printer",
            PrinterGCodeSourceOptions::DECLARATION_ORDER.as_slice(),
            GCodeOptionSource::Printer,
        ),
        (
            "process",
            ProcessGCodeSourceOptions::DECLARATION_ORDER.as_slice(),
            GCodeOptionSource::Process,
        ),
        (
            "filament",
            FilamentGCodeSourceOptions::DECLARATION_ORDER.as_slice(),
            GCodeOptionSource::Filament,
        ),
        (
            "residual",
            ProjectGCodeSourceOptions::DECLARATION_ORDER.as_slice(),
            GCodeOptionSource::Project,
        ),
    ];
    let mut source_union = BTreeSet::new();
    for (scope, declaration_order, owner) in expected_sources {
        let row_keys = rows
            .iter()
            .filter(|row| row.raw_scope == scope)
            .map(|row| row.key.as_str())
            .collect::<BTreeSet<_>>();
        let declaration_keys = declaration_order.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(row_keys, declaration_keys, "{scope}");
        for key in declaration_order {
            assert!(source_union.insert(*key), "duplicate source key {key}");
        }
        assert_eq!(
            GCodeOptions::FIELD_METADATA
                .iter()
                .filter(|(_, _, source)| *source == owner)
                .map(|(_, key, _)| *key)
                .collect::<BTreeSet<_>>(),
            declaration_keys,
            "{scope} ledger owner"
        );
    }

    let inventory_keys = rows
        .iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(source_union, inventory_keys);
    let expected_metadata = EXPECTED_HPP_ORDER.map(|key| {
        let row = rows.iter().find(|row| row.key == key).unwrap();
        let field = match key {
            "required_nozzle_HRC" => "required_nozzle_hrc",
            key => key,
        };
        let source = match row.raw_scope.as_str() {
            "printer" => GCodeOptionSource::Printer,
            "process" => GCodeOptionSource::Process,
            "filament" => GCodeOptionSource::Filament,
            "residual" => GCodeOptionSource::Project,
            scope => panic!("unexpected raw scope {scope}"),
        };
        (field, key, source)
    });
    assert_eq!(GCodeOptions::FIELD_METADATA, expected_metadata);
    assert_eq!(
        GCodeOptions::DECLARATION_ORDER
            .into_iter()
            .collect::<BTreeSet<_>>(),
        inventory_keys
    );
    assert_eq!(GCodeOptions::DECLARATION_ORDER.len(), 149);
    assert_eq!(GCodeOptions::FIELD_METADATA.len(), 149);
    assert_eq!(
        GCodeOptions::FIELD_METADATA.map(|(_, key, _)| key),
        GCodeOptions::DECLARATION_ORDER
    );
    assert_eq!(
        GCodeOptions::FIELD_METADATA
            .iter()
            .map(|(field, _, _)| *field)
            .collect::<BTreeSet<_>>()
            .len(),
        149
    );
    assert!(
        GCodeOptions::FIELD_METADATA.contains(&(
            "required_nozzle_hrc",
            "required_nozzle_HRC",
            GCodeOptionSource::Filament,
        ))
    );
    for excluded in ["thumbnail_size", "bbl_bed_temperature_gcode"] {
        assert!(!inventory_keys.contains(excluded));
        assert!(!GCodeOptions::DECLARATION_ORDER.contains(&excluded));
    }
}

#[test]
fn gcode_options_inventory_has_exact_types_shapes_and_nullability() {
    let inventory = inventory();
    let rows = gcode_rows(&inventory);
    let histogram = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
        counts
    });
    assert_eq!(
        histogram,
        BTreeMap::from([
            ("coBool", 27),
            ("coBools", 9),
            ("coEnum", 6),
            ("coEnums", 5),
            ("coFloat", 14),
            ("coFloats", 38),
            ("coFloatOrPercent", 3),
            ("coInt", 5),
            ("coInts", 11),
            ("coPercent", 1),
            ("coPercents", 1),
            ("coPoints", 1),
            ("coString", 13),
            ("coStrings", 15),
        ])
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.wire_shape == "scalar_string")
            .count(),
        69
    );
    assert_eq!(
        rows.iter().filter(|row| row.wire_shape == "array").count(),
        80
    );
    assert!(
        rows.iter()
            .all(|row| matches!(row.wire_shape.as_str(), "scalar_string" | "array"))
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.nullable)
            .map(|row| row.key.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "filament_adaptive_volumetric_speed",
            "filament_cooling_before_tower",
            "filament_flow_ratio",
            "filament_flush_temp",
            "filament_flush_volumetric_speed",
            "long_retractions_when_ec",
            "nozzle_flush_dataset",
            "nozzle_type",
            "retraction_distances_when_ec",
        ])
    );
}
