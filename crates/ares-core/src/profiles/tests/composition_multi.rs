use crate::{
    FilamentGCodeSourceOptions, FilamentOptions, FilamentPrintSourceOptions,
    FilamentRegionSourceOptions, FilamentRetractOverrideOptions, MergedProfile, ProfileKind,
    compose_profile_fragments, merge_profile_fragments,
};

use super::{fragments, selection};

#[test]
fn two_filaments_append_every_representative_concrete_type_in_selection_order() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p"}"#,
        br#"{
            "type":"filament","name":"a",
            "filament_diameter":[1.7],
            "enable_pressure_advance":[true],
            "filament_type":["PLA"],
            "filament_flow_ratio":[0.91,"nil"],
            "filament_notes":[],
            "filament_extruder_variant":["Direct Drive Standard","Direct Drive High Flow"],
            "adaptive_pressure_advance_model":["0,0,0"],
            "pellet_flow_coefficient":[0.4]
        }"#,
        br#"{
            "type":"filament","name":"b","filament_id":"B-ID",
            "filament_diameter":[1.8,1.9],
            "enable_pressure_advance":[false,true],
            "filament_type":["PETG","PCTG"],
            "filament_flow_ratio":["nil",1.02],
            "filament_notes":["note-b"],
            "filament_extruder_variant":["Bowden Standard"],
            "adaptive_pressure_advance_model":["1,1,1","2,2,2"],
            "filament_z_hop_types":["nil"],
            "pellet_flow_coefficient":[0.5,0.6]
        }"#,
        br#"{
            "type":"filament","name":"joined",
            "filament_diameter":[1.7,1.8,1.9],
            "enable_pressure_advance":[true,false,true],
            "filament_type":["PLA","PETG","PCTG"],
            "filament_flow_ratio":[0.91,"nil","nil",1.02],
            "filament_notes":["note-b"],
            "filament_extruder_variant":["Direct Drive Standard","Direct Drive High Flow","Bowden Standard"],
            "adaptive_pressure_advance_model":["0,0,0","1,1,1","2,2,2"],
            "pellet_flow_coefficient":[0.4,0.5,0.6]
        }"#,
    ]);

    let composed = compose_profile_fragments(&fragments, &selection("p", "m", ["a", "b"])).unwrap();
    let actual = &composed.settings().filament;
    let expected = merged_filament(&fragments, "joined");

    assert_eq!(
        actual.gcode.filament_diameter,
        expected.gcode.filament_diameter
    );
    assert_eq!(
        actual.gcode.enable_pressure_advance,
        expected.gcode.enable_pressure_advance
    );
    assert_eq!(actual.gcode.filament_type, expected.gcode.filament_type);
    assert_eq!(
        actual.gcode.filament_flow_ratio,
        expected.gcode.filament_flow_ratio
    );
    assert_eq!(actual.print.filament_notes, expected.print.filament_notes);
    assert_eq!(
        actual.gcode.filament_extruder_variant,
        expected.gcode.filament_extruder_variant
    );
    assert_eq!(
        actual.gcode.adaptive_pressure_advance_model,
        expected.gcode.adaptive_pressure_advance_model
    );
    assert_eq!(
        actual.pellet_flow_coefficient,
        expected.pellet_flow_coefficient
    );

    let left = merged_filament(&fragments, "a");
    let right = merged_filament(&fragments, "b");
    let enum_values = &actual.retract_overrides.filament_z_hop_types;
    let split = left.retract_overrides.filament_z_hop_types.len();
    assert_eq!(
        &enum_values[..split],
        left.retract_overrides.filament_z_hop_types.as_slice()
    );
    assert_eq!(
        &enum_values[split..],
        right.retract_overrides.filament_z_hop_types.as_slice()
    );

    assert_eq!(composed.filament_names(), ["a", "b"]);
    assert_eq!(
        composed.settings().project.preset.filament_settings_id.0,
        ["a", "b"]
    );
    assert_eq!(
        int_values(&composed.settings().project.gcode.filament_map),
        [1, 1]
    );
    assert_eq!(
        composed.settings().project.gcode.filament_ids.0,
        ["", "B-ID"]
    );
    assert_eq!(
        int_values(&composed.settings().project.preset.filament_self_index),
        [1, 1, 2]
    );
}

#[test]
fn fixed_filament_owner_exhaustively_names_all_122_vector_like_fields() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p"}"#,
        br#"{"type":"filament","name":"f"}"#,
    ]);
    let composed = compose_profile_fragments(&fragments, &selection("p", "m", ["f"])).unwrap();

    assert_fixed_filament_owner(&composed.settings().filament);
    assert_eq!(53 + 48 + 4 + 16 + 1, 122);
}

fn merged_filament(fragments: &[crate::ProfileFragment], name: &str) -> FilamentOptions {
    match merge_profile_fragments(fragments, ProfileKind::Filament, name).unwrap() {
        MergedProfile::Filament { options, .. } => options,
        other => panic!("expected filament result, got {other:?}"),
    }
}

fn int_values(values: &crate::OrcaInts) -> Vec<i32> {
    values.0.iter().map(|value| value.0).collect()
}

fn assert_fixed_filament_owner(options: &FilamentOptions) {
    let FilamentOptions {
        gcode,
        print,
        region,
        retract_overrides,
        pellet_flow_coefficient: _,
    } = options;
    let FilamentGCodeSourceOptions {
        filament_end_gcode: _,
        filament_flow_ratio: _,
        enable_pressure_advance: _,
        pressure_advance: _,
        adaptive_pressure_advance: _,
        adaptive_pressure_advance_overhangs: _,
        adaptive_pressure_advance_model: _,
        adaptive_pressure_advance_bridges: _,
        filament_diameter: _,
        filament_adaptive_volumetric_speed: _,
        volumetric_speed_coefficients: _,
        filament_adhesiveness_category: _,
        filament_density: _,
        filament_type: _,
        filament_soluble: _,
        filament_colour: _,
        filament_vendor: _,
        filament_is_support: _,
        filament_printable: _,
        filament_change_length: _,
        filament_cost: _,
        default_filament_colour: _,
        temperature_vitrification: _,
        filament_max_volumetric_speed: _,
        required_nozzle_hrc: _,
        filament_extruder_variant: _,
        filament_flush_volumetric_speed: _,
        filament_flush_temp: _,
        retraction_distances_when_ec: _,
        long_retractions_when_ec: _,
        filament_start_gcode: _,
        filament_change_extrusion_role_gcode: _,
        filament_loading_speed: _,
        filament_loading_speed_start: _,
        filament_unloading_speed: _,
        filament_unloading_speed_start: _,
        filament_toolchange_delay: _,
        filament_cooling_moves: _,
        filament_cooling_initial_speed: _,
        filament_minimal_purge_on_wipe_tower: _,
        filament_cooling_before_tower: _,
        filament_tower_interface_pre_extrusion_dist: _,
        filament_tower_interface_pre_extrusion_length: _,
        filament_tower_ironing_area: _,
        filament_tower_interface_purge_volume: _,
        filament_tower_interface_print_temp: _,
        filament_cooling_final_speed: _,
        filament_ramming_parameters: _,
        filament_multitool_ramming: _,
        filament_multitool_ramming_volume: _,
        filament_multitool_ramming_flow: _,
        filament_stamping_loading_speed: _,
        filament_stamping_distance: _,
    } = gcode;
    let FilamentPrintSourceOptions {
        additional_cooling_fan_speed: _,
        close_additional_fan_first_x_layers: _,
        additional_fan_full_speed_layer: _,
        first_x_layer_fan_speed: _,
        cool_plate_temp: _,
        textured_cool_plate_temp: _,
        supertack_plate_temp: _,
        eng_plate_temp: _,
        hot_plate_temp: _,
        textured_plate_temp: _,
        supertack_plate_temp_initial_layer: _,
        cool_plate_temp_initial_layer: _,
        textured_cool_plate_temp_initial_layer: _,
        eng_plate_temp_initial_layer: _,
        hot_plate_temp_initial_layer: _,
        textured_plate_temp_initial_layer: _,
        enable_overhang_bridge_fan: _,
        overhang_fan_speed: _,
        overhang_fan_threshold: _,
        slow_down_for_layer_cooling: _,
        close_fan_the_first_x_layers: _,
        reduce_fan_stop_start_freq: _,
        dont_slow_down_outer_wall: _,
        fan_cooling_layer_time: _,
        activate_air_filtration: _,
        activate_air_filtration_during_print: _,
        activate_air_filtration_on_completion: _,
        during_print_exhaust_fan_speed: _,
        complete_print_exhaust_fan_speed: _,
        nozzle_temperature_initial_layer: _,
        full_fan_speed_layer: _,
        fan_max_speed: _,
        fan_min_speed: _,
        slow_down_min_speed: _,
        slow_down_layer_time: _,
        nozzle_temperature: _,
        nozzle_temperature_range_low: _,
        nozzle_temperature_range_high: _,
        idle_temperature: _,
        filament_shrink: _,
        filament_shrinkage_compensation_z: _,
        support_material_interface_fan_speed: _,
        internal_bridge_fan_speed: _,
        ironing_fan_speed: _,
        filament_notes: _,
        activate_chamber_temp_control: _,
        chamber_temperature: _,
        chamber_minimal_temperature: _,
    } = print;
    let FilamentRegionSourceOptions {
        filament_ironing_flow: _,
        filament_ironing_spacing: _,
        filament_ironing_inset: _,
        filament_ironing_speed: _,
    } = region;
    let FilamentRetractOverrideOptions {
        filament_retraction_length: _,
        filament_z_hop: _,
        filament_z_hop_types: _,
        filament_retract_lift_above: _,
        filament_retract_lift_below: _,
        filament_retract_lift_enforce: _,
        filament_retraction_speed: _,
        filament_deretraction_speed: _,
        filament_retract_restart_extra: _,
        filament_retraction_minimum_travel: _,
        filament_wipe_distance: _,
        filament_retract_when_changing_layer: _,
        filament_wipe: _,
        filament_retract_before_wipe: _,
        filament_long_retractions_when_cut: _,
        filament_retraction_distances_when_cut: _,
    } = retract_overrides;
}
