use super::*;

#[test]
fn gcode_options_projection_preserves_each_filament_field() {
    assert_filament_projection!(
        filament_end_gcode,
        OrcaStrings(vec!["filament-end-a".into(), "filament-end-b".into()])
    );
    assert_filament_projection!(
        filament_flow_ratio,
        vec![Nullable::Nil, Nullable::Value(OrcaFloat(9301.01))]
    );
    assert_filament_projection!(
        enable_pressure_advance,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        pressure_advance,
        OrcaFloats(vec![OrcaFloat(9302.02), OrcaFloat(9303.03)])
    );
    assert_filament_projection!(
        adaptive_pressure_advance,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        adaptive_pressure_advance_overhangs,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        adaptive_pressure_advance_model,
        CsvTable(vec!["filament-model-a".into(), "filament-model-b".into()])
    );
    assert_filament_projection!(
        adaptive_pressure_advance_bridges,
        OrcaFloats(vec![OrcaFloat(9304.04), OrcaFloat(9305.05)])
    );
    assert_filament_projection!(
        filament_diameter,
        OrcaFloats(vec![OrcaFloat(9306.06), OrcaFloat(9307.07)])
    );
    assert_filament_projection!(
        filament_adaptive_volumetric_speed,
        vec![Nullable::Nil, Nullable::Value(OrcaBool(true))]
    );
    assert_filament_projection!(
        volumetric_speed_coefficients,
        SpaceTuple(vec!["filament-coefficients-a".into(), "filament-coefficients-b".into()])
    );
    assert_filament_projection!(
        filament_adhesiveness_category,
        OrcaInts(vec![OrcaInt(9308), OrcaInt(9309)])
    );
    assert_filament_projection!(
        filament_density,
        OrcaFloats(vec![OrcaFloat(9310.10), OrcaFloat(9311.11)])
    );
    assert_filament_projection!(
        filament_type,
        OrcaStrings(vec!["filament-type-a".into(), "filament-type-b".into()])
    );
    assert_filament_projection!(
        filament_soluble,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        filament_colour,
        OrcaStrings(vec!["filament-colour-a".into(), "filament-colour-b".into()])
    );
    assert_filament_projection!(
        filament_vendor,
        OrcaStrings(vec!["filament-vendor-a".into(), "filament-vendor-b".into()])
    );
    assert_filament_projection!(
        filament_is_support,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        filament_printable,
        OrcaInts(vec![OrcaInt(9312), OrcaInt(9313)])
    );
    assert_filament_projection!(
        filament_change_length,
        OrcaFloats(vec![OrcaFloat(9314.14), OrcaFloat(9315.15)])
    );
    assert_filament_projection!(
        filament_cost,
        OrcaFloats(vec![OrcaFloat(9316.16), OrcaFloat(9317.17)])
    );
    assert_filament_projection!(
        default_filament_colour,
        OrcaStrings(vec!["filament-default-a".into(), "filament-default-b".into()])
    );
    assert_filament_projection!(
        temperature_vitrification,
        OrcaInts(vec![OrcaInt(9318), OrcaInt(9319)])
    );
    assert_filament_projection!(
        filament_max_volumetric_speed,
        OrcaFloats(vec![OrcaFloat(9320.20), OrcaFloat(9321.21)])
    );
    assert_filament_projection!(
        required_nozzle_hrc,
        OrcaInts(vec![OrcaInt(9322), OrcaInt(9323)])
    );
    assert_filament_projection!(
        filament_extruder_variant,
        VariantStride(vec!["filament-variant-a".into(), "filament-variant-b".into()])
    );
    assert_filament_projection!(
        filament_flush_volumetric_speed,
        vec![Nullable::Nil, Nullable::Value(OrcaFloat(9324.24))]
    );
    assert_filament_projection!(
        filament_flush_temp,
        vec![Nullable::Nil, Nullable::Value(OrcaInt(9325))]
    );
    assert_filament_projection!(
        retraction_distances_when_ec,
        vec![Nullable::Nil, Nullable::Value(OrcaFloat(9326.26))]
    );
    assert_filament_projection!(
        long_retractions_when_ec,
        vec![Nullable::Nil, Nullable::Value(OrcaBool(true))]
    );
    assert_filament_projection!(
        filament_start_gcode,
        OrcaStrings(vec!["filament-start-a".into(), "filament-start-b".into()])
    );
    assert_filament_projection!(
        filament_change_extrusion_role_gcode,
        OrcaStrings(vec!["filament-role-a".into(), "filament-role-b".into()])
    );
    assert_filament_projection!(
        filament_loading_speed,
        OrcaFloats(vec![OrcaFloat(9327.27), OrcaFloat(9328.28)])
    );
    assert_filament_projection!(
        filament_loading_speed_start,
        OrcaFloats(vec![OrcaFloat(9329.29), OrcaFloat(9330.30)])
    );
    assert_filament_projection!(
        filament_unloading_speed,
        OrcaFloats(vec![OrcaFloat(9331.31), OrcaFloat(9332.32)])
    );
    assert_filament_projection!(
        filament_unloading_speed_start,
        OrcaFloats(vec![OrcaFloat(9333.33), OrcaFloat(9334.34)])
    );
    assert_filament_projection!(
        filament_toolchange_delay,
        OrcaFloats(vec![OrcaFloat(9335.35), OrcaFloat(9336.36)])
    );
    assert_filament_projection!(
        filament_cooling_moves,
        OrcaInts(vec![OrcaInt(9337), OrcaInt(9338)])
    );
    assert_filament_projection!(
        filament_cooling_initial_speed,
        OrcaFloats(vec![OrcaFloat(9339.39), OrcaFloat(9340.40)])
    );
    assert_filament_projection!(
        filament_minimal_purge_on_wipe_tower,
        OrcaFloats(vec![OrcaFloat(9341.41), OrcaFloat(9342.42)])
    );
    assert_filament_projection!(
        filament_cooling_before_tower,
        vec![Nullable::Nil, Nullable::Value(OrcaFloat(9343.43))]
    );
    assert_filament_projection!(
        filament_tower_interface_pre_extrusion_dist,
        OrcaFloats(vec![OrcaFloat(9344.44), OrcaFloat(9345.45)])
    );
    assert_filament_projection!(
        filament_tower_interface_pre_extrusion_length,
        OrcaFloats(vec![OrcaFloat(9346.46), OrcaFloat(9347.47)])
    );
    assert_filament_projection!(
        filament_tower_ironing_area,
        OrcaFloats(vec![OrcaFloat(9348.48), OrcaFloat(9349.49)])
    );
    assert_filament_projection!(
        filament_tower_interface_purge_volume,
        OrcaFloats(vec![OrcaFloat(9350.50), OrcaFloat(9351.51)])
    );
    assert_filament_projection!(
        filament_tower_interface_print_temp,
        OrcaInts(vec![OrcaInt(9352), OrcaInt(9353)])
    );
    assert_filament_projection!(
        filament_cooling_final_speed,
        OrcaFloats(vec![OrcaFloat(9354.54), OrcaFloat(9355.55)])
    );
    assert_filament_projection!(
        filament_ramming_parameters,
        RammingParameters(vec!["filament-ramming-a".into(), "filament-ramming-b".into()])
    );
    assert_filament_projection!(
        filament_multitool_ramming,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_filament_projection!(
        filament_multitool_ramming_volume,
        OrcaFloats(vec![OrcaFloat(9356.56), OrcaFloat(9357.57)])
    );
    assert_filament_projection!(
        filament_multitool_ramming_flow,
        OrcaFloats(vec![OrcaFloat(9358.58), OrcaFloat(9359.59)])
    );
    assert_filament_projection!(
        filament_stamping_loading_speed,
        OrcaFloats(vec![OrcaFloat(9360.60), OrcaFloat(9361.61)])
    );
    assert_filament_projection!(
        filament_stamping_distance,
        OrcaFloats(vec![OrcaFloat(9362.62), OrcaFloat(9363.63)])
    );
}
