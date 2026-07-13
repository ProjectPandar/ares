use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions, Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats,
    OrcaInt, OrcaInts, OrcaPercents, OrcaStrings, Percent, RawOverhangFanThreshold,
    RetractLiftEnforce, ZHopType,
};

pub(super) fn assert_concrete_types(value: &FilamentOptions) {
    assert_print_types(&value.print);
    assert_region_types(&value.region);
    assert_retract_types(&value.retract_overrides);
    let _: &OrcaFloats = &value.pellet_flow_coefficient;
}

fn assert_print_types(value: &FilamentPrintSourceOptions) {
    fields!(value, OrcaBools;
        activate_air_filtration, activate_air_filtration_during_print,
        activate_air_filtration_on_completion, activate_chamber_temp_control,
        dont_slow_down_outer_wall, enable_overhang_bridge_fan, reduce_fan_stop_start_freq,
        slow_down_for_layer_cooling
    );
    fields!(value, OrcaFloats;
        fan_cooling_layer_time, fan_max_speed, fan_min_speed, first_x_layer_fan_speed,
        slow_down_layer_time, slow_down_min_speed
    );
    fields!(value, OrcaInts;
        additional_cooling_fan_speed, additional_fan_full_speed_layer,
        chamber_minimal_temperature, chamber_temperature, close_additional_fan_first_x_layers,
        close_fan_the_first_x_layers, complete_print_exhaust_fan_speed, cool_plate_temp,
        cool_plate_temp_initial_layer, during_print_exhaust_fan_speed, eng_plate_temp,
        eng_plate_temp_initial_layer, full_fan_speed_layer, hot_plate_temp,
        hot_plate_temp_initial_layer, idle_temperature, internal_bridge_fan_speed,
        ironing_fan_speed, nozzle_temperature, nozzle_temperature_initial_layer,
        nozzle_temperature_range_high, nozzle_temperature_range_low, overhang_fan_speed,
        supertack_plate_temp, supertack_plate_temp_initial_layer,
        support_material_interface_fan_speed, textured_cool_plate_temp,
        textured_cool_plate_temp_initial_layer, textured_plate_temp,
        textured_plate_temp_initial_layer
    );
    fields!(value, OrcaPercents; filament_shrink, filament_shrinkage_compensation_z);
    let _: &Vec<RawOverhangFanThreshold> = &value.overhang_fan_threshold;
    let _: &OrcaStrings = &value.filament_notes;
}

fn assert_region_types(value: &FilamentRegionSourceOptions) {
    let _: &Vec<Nullable<Percent>> = &value.filament_ironing_flow;
    fields!(value, Vec<Nullable<OrcaFloat>>;
        filament_ironing_inset, filament_ironing_spacing, filament_ironing_speed
    );
}

fn assert_retract_types(value: &FilamentRetractOverrideOptions) {
    fields!(value, Vec<Nullable<OrcaBool>>;
        filament_long_retractions_when_cut, filament_retract_when_changing_layer,
        filament_wipe
    );
    fields!(value, Vec<Nullable<OrcaFloat>>;
        filament_deretraction_speed, filament_retract_lift_above,
        filament_retract_lift_below, filament_retract_restart_extra,
        filament_retraction_distances_when_cut, filament_retraction_length,
        filament_retraction_minimum_travel, filament_retraction_speed,
        filament_wipe_distance, filament_z_hop
    );
    let _: &Vec<Nullable<Percent>> = &value.filament_retract_before_wipe;
    let _: &Vec<Nullable<RetractLiftEnforce>> = &value.filament_retract_lift_enforce;
    let _: &Vec<Nullable<ZHopType>> = &value.filament_z_hop_types;
}

#[test]
fn every_remaining_public_field_has_its_concrete_type() {
    assert_concrete_types(&FilamentOptions::default());
    let _: &OrcaInt = &FilamentOptions::default().print.additional_cooling_fan_speed.0[0];
}

macro_rules! fields {
    ($value:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(let _: &$ty = &$value.$field;)+
    };
}

use fields;
