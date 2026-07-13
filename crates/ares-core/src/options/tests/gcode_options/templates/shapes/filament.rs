use super::*;

pub(super) fn verify_arrays() -> Vec<&'static str> {
    verify_array_fields! {
        filament;
        filament_end_gcode => "filament_end_gcode" = OrcaStrings(Vec::new()),
        filament_flow_ratio => "filament_flow_ratio" = nullable_floats(&[
            None, Some(2001.1), None, Some(2003.3), Some(2004.4), None, Some(2006.6), None,
        ]),
        enable_pressure_advance => "enable_pressure_advance" = bools(&[true]),
        pressure_advance => "pressure_advance" = floats(&[2011.1, 2012.2, 2013.3]),
        adaptive_pressure_advance => "adaptive_pressure_advance" = bools(&[false, true]),
        adaptive_pressure_advance_overhangs => "adaptive_pressure_advance_overhangs" =
            bools(&[true, false, true]),
        adaptive_pressure_advance_model => "adaptive_pressure_advance_model" =
            CsvTable(owned_strings(&["2021,2022|a", "2023\\b\n"])),
        adaptive_pressure_advance_bridges => "adaptive_pressure_advance_bridges" =
            floats(&[2031.1, 2032.2]),
        filament_diameter => "filament_diameter" = floats(&[2041.1, 2042.2]),
        filament_adaptive_volumetric_speed => "filament_adaptive_volumetric_speed" =
            nullable_bools(&[
                None, Some(true), Some(false), None, Some(false), Some(true), None, Some(true),
            ]),
        volumetric_speed_coefficients => "volumetric_speed_coefficients" = SpaceTuple(
            owned_strings(&["2050", "2051", "2052", "2053", "2054", "2055", "2056", "2057"]),
        ),
        filament_adhesiveness_category => "filament_adhesiveness_category" = ints(&[2061, 2062]),
        filament_density => "filament_density" = floats(&[2071.1, 2072.2]),
        filament_type => "filament_type" = strings(&["type-2081", "type-2082"]),
        filament_soluble => "filament_soluble" = bools(&[true, false, false, true]),
        filament_colour => "filament_colour" = strings(&["#209101", "#209202"]),
        filament_vendor => "filament_vendor" = strings(&["vendor-2101", "vendor-2102"]),
        filament_is_support => "filament_is_support" =
            bools(&[false, true, false, false, true]),
        filament_printable => "filament_printable" = ints(&[2111, 2112]),
        filament_change_length => "filament_change_length" = floats(&[2121.1, 2122.2]),
        filament_cost => "filament_cost" = floats(&[2131.1, 2132.2]),
        default_filament_colour => "default_filament_colour" =
            strings(&["#214101", "#214202"]),
        temperature_vitrification => "temperature_vitrification" = ints(&[2151, 2152]),
        filament_max_volumetric_speed => "filament_max_volumetric_speed" =
            floats(&[2160.0, 2161.1, 2162.2, 2163.3, 2164.4, 2165.5, 2166.6, 2167.7]),
        required_nozzle_hrc => "required_nozzle_HRC" = ints(&[2171, 2172]),
        filament_extruder_variant => "filament_extruder_variant" = VariantStride(
            owned_strings(&["v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7"]),
        ),
        filament_flush_volumetric_speed => "filament_flush_volumetric_speed" = nullable_floats(&[
            Some(2180.0), None, Some(2182.2), None, Some(2184.4), None, None, Some(2187.7),
        ]),
        filament_flush_temp => "filament_flush_temp" = nullable_ints(&[
            None, Some(2191), Some(2192), None, Some(2194), None, Some(2196), None,
        ]),
        retraction_distances_when_ec => "retraction_distances_when_ec" = nullable_floats(&[
            None, Some(2201.1), None, Some(2203.3), None, Some(2205.5), Some(2206.6), None,
        ]),
        long_retractions_when_ec => "long_retractions_when_ec" = nullable_bools(&[
            Some(true), None, Some(false), None, Some(true), None, Some(false), None,
        ]),
        filament_start_gcode => "filament_start_gcode" =
            strings(&["M701 {filament[0]}\n", "M702\\path\r\n"]),
        filament_change_extrusion_role_gcode => "filament_change_extrusion_role_gcode" =
            strings(&["role-2231", "role-2232"]),
        filament_loading_speed => "filament_loading_speed" = floats(&[2241.1, 2242.2]),
        filament_loading_speed_start => "filament_loading_speed_start" =
            floats(&[2251.1, 2252.2]),
        filament_unloading_speed => "filament_unloading_speed" = floats(&[2261.1, 2262.2]),
        filament_unloading_speed_start => "filament_unloading_speed_start" =
            floats(&[2271.1, 2272.2]),
        filament_toolchange_delay => "filament_toolchange_delay" = floats(&[2281.1, 2282.2]),
        filament_cooling_moves => "filament_cooling_moves" = ints(&[2291, 2292]),
        filament_cooling_initial_speed => "filament_cooling_initial_speed" =
            floats(&[2301.1, 2302.2]),
        filament_minimal_purge_on_wipe_tower => "filament_minimal_purge_on_wipe_tower" =
            floats(&[2311.1, 2312.2]),
        filament_cooling_before_tower => "filament_cooling_before_tower" = nullable_floats(&[
            None, Some(2321.1), Some(2322.2), None, None, Some(2325.5), None, Some(2327.7),
        ]),
        filament_tower_interface_pre_extrusion_dist =>
            "filament_tower_interface_pre_extrusion_dist" = floats(&[2331.1, 2332.2]),
        filament_tower_interface_pre_extrusion_length =>
            "filament_tower_interface_pre_extrusion_length" = floats(&[2341.1, 2342.2]),
        filament_tower_ironing_area => "filament_tower_ironing_area" = floats(&[2351.1, 2352.2]),
        filament_tower_interface_purge_volume => "filament_tower_interface_purge_volume" =
            floats(&[2361.1, 2362.2]),
        filament_tower_interface_print_temp => "filament_tower_interface_print_temp" =
            ints(&[2371, 2372]),
        filament_cooling_final_speed => "filament_cooling_final_speed" =
            floats(&[2381.1, 2382.2]),
        filament_ramming_parameters => "filament_ramming_parameters" =
            RammingParameters(owned_strings(&["2391 2392|a", "2393;2394\\b"])),
        filament_multitool_ramming => "filament_multitool_ramming" =
            bools(&[true, false, true, true, false, false]),
        filament_multitool_ramming_volume => "filament_multitool_ramming_volume" =
            floats(&[2411.1, 2412.2]),
        filament_multitool_ramming_flow => "filament_multitool_ramming_flow" =
            floats(&[2421.1, 2422.2]),
        filament_stamping_loading_speed => "filament_stamping_loading_speed" =
            floats(&[2431.1, 2432.2]),
        filament_stamping_distance => "filament_stamping_distance" = floats(&[2441.1, 2442.2]),
    }
}
