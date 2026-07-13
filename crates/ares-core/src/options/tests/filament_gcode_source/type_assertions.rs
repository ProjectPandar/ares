use super::super::super::{
    CsvTable, FilamentGCodeSourceOptions, Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats,
    OrcaInt, OrcaInts, OrcaStrings, RammingParameters, SpaceTuple, VariantStride,
};

pub(super) fn assert_concrete_types(value: &FilamentGCodeSourceOptions) {
    fields!(value, OrcaBools;
        adaptive_pressure_advance, adaptive_pressure_advance_overhangs,
        enable_pressure_advance, filament_is_support, filament_multitool_ramming,
        filament_soluble
    );
    fields!(value, OrcaFloats;
        adaptive_pressure_advance_bridges, filament_change_length, filament_cooling_final_speed,
        filament_cooling_initial_speed, filament_cost, filament_density, filament_diameter,
        filament_loading_speed, filament_loading_speed_start, filament_max_volumetric_speed,
        filament_minimal_purge_on_wipe_tower, filament_multitool_ramming_flow,
        filament_multitool_ramming_volume, filament_stamping_distance,
        filament_stamping_loading_speed, filament_toolchange_delay,
        filament_tower_interface_pre_extrusion_dist,
        filament_tower_interface_pre_extrusion_length,
        filament_tower_interface_purge_volume, filament_tower_ironing_area,
        filament_unloading_speed, filament_unloading_speed_start, pressure_advance
    );
    fields!(value, OrcaInts;
        filament_adhesiveness_category, filament_cooling_moves, filament_printable,
        filament_tower_interface_print_temp, required_nozzle_hrc, temperature_vitrification
    );
    fields!(value, OrcaStrings;
        default_filament_colour, filament_change_extrusion_role_gcode, filament_colour,
        filament_end_gcode, filament_start_gcode, filament_type, filament_vendor
    );
    let _: &CsvTable = &value.adaptive_pressure_advance_model;
    let _: &VariantStride = &value.filament_extruder_variant;
    let _: &RammingParameters = &value.filament_ramming_parameters;
    let _: &SpaceTuple = &value.volumetric_speed_coefficients;
    fields!(value, Vec<Nullable<OrcaBool>>;
        filament_adaptive_volumetric_speed, long_retractions_when_ec
    );
    fields!(value, Vec<Nullable<OrcaFloat>>;
        filament_cooling_before_tower, filament_flow_ratio,
        filament_flush_volumetric_speed, retraction_distances_when_ec
    );
    let _: &Vec<Nullable<OrcaInt>> = &value.filament_flush_temp;
}

macro_rules! fields {
    ($value:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(let _: &$ty = &$value.$field;)+
    };
}

use fields;
