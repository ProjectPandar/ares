#[test]
fn exposes_public_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "activate_air_filtration",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "activate_air_filtration_during_print",
            crate::OptionValueKind::Bools,
            "true",
        ),
        (
            "activate_air_filtration_on_completion",
            crate::OptionValueKind::Bools,
            "true",
        ),
        (
            "close_fan_the_first_x_layers",
            crate::OptionValueKind::Ints,
            "1",
        ),
        (
            "complete_print_exhaust_fan_speed",
            crate::OptionValueKind::Ints,
            "80",
        ),
        ("default_acceleration", crate::OptionValueKind::Float, "500"),
        (
            "default_filament_profile",
            crate::OptionValueKind::Strings,
            "",
        ),
        ("default_print_profile", crate::OptionValueKind::String, ""),
        (
            "during_print_exhaust_fan_speed",
            crate::OptionValueKind::Ints,
            "60",
        ),
        (
            "slow_down_for_layer_cooling",
            crate::OptionValueKind::Bools,
            "true",
        ),
        ("bridge_no_support", crate::OptionValueKind::Bool, "false"),
        (
            "dont_filter_internal_bridges",
            crate::OptionValueKind::Enum,
            "disabled",
        ),
        (
            "enable_extra_bridge_layer",
            crate::OptionValueKind::Enum,
            "disabled",
        ),
        ("max_bridge_length", crate::OptionValueKind::Float, "10"),
        ("thick_bridges", crate::OptionValueKind::Bool, "false"),
        (
            "thick_internal_bridges",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "bottom_surface_pattern",
            crate::OptionValueKind::Enum,
            "monotonic",
        ),
        (
            "ensure_vertical_shell_thickness",
            crate::OptionValueKind::Enum,
            "ensure_all",
        ),
        ("filament_end_gcode", crate::OptionValueKind::Strings, " "),
        (
            "internal_solid_infill_pattern",
            crate::OptionValueKind::Enum,
            "monotonic",
        ),
        (
            "machine_end_gcode",
            crate::OptionValueKind::String,
            "M104 S0 ; turn off temperature\nG28 X0  ; home X axis\nM84     ; disable motors\n",
        ),
        (
            "printing_by_object_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "top_surface_pattern",
            crate::OptionValueKind::Enum,
            "monotonicline",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    let definition = crate::option_definition("bridge_speed").unwrap();

    assert_eq!(definition.kind, crate::OptionValueKind::Float);

    let printable_area = crate::option_definition("printable_area").unwrap();
    assert_eq!(printable_area.kind, crate::OptionValueKind::Points);

    let bed_model = crate::option_definition("bed_custom_model").unwrap();
    assert_eq!(bed_model.kind, crate::OptionValueKind::String);
    assert_eq!(bed_model.default_value, "");

    let auth = crate::option_definition("printhost_authorization_type").unwrap();
    assert_eq!(auth.kind, crate::OptionValueKind::Enum);
    assert_eq!(auth.default_value, "key");

    let preset_names = crate::option_definition("preset_names").unwrap();
    assert_eq!(preset_names.kind, crate::OptionValueKind::Strings);

    let reduce = crate::option_definition("reduce_crossing_wall").unwrap();
    assert_eq!(reduce.kind, crate::OptionValueKind::Bool);
    assert_eq!(reduce.default_value, "false");

    let max_detour = crate::option_definition("max_travel_detour_distance").unwrap();
    assert_eq!(max_detour.kind, crate::OptionValueKind::FloatOrPercent);
    assert_eq!(max_detour.default_value, "0");

    let cool_plate = crate::option_definition("cool_plate_temp").unwrap();
    assert_eq!(cool_plate.kind, crate::OptionValueKind::Ints);
    assert_eq!(cool_plate.default_value, "35");

    let hot_plate = crate::option_definition("hot_plate_temp").unwrap();
    assert_eq!(hot_plate.kind, crate::OptionValueKind::Ints);
    assert_eq!(hot_plate.default_value, "45");

    for (key, default_value) in [
        ("cool_plate_temp_initial_layer", "35"),
        ("eng_plate_temp_initial_layer", "45"),
        ("hot_plate_temp_initial_layer", "45"),
        ("supertack_plate_temp_initial_layer", "35"),
        ("textured_cool_plate_temp_initial_layer", "40"),
        ("textured_plate_temp_initial_layer", "45"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Ints);
        assert_eq!(definition.default_value, default_value);
    }

    for (key, kind, default_value) in [
        (
            "counterbore_hole_bridging",
            crate::OptionValueKind::Enum,
            "none",
        ),
        ("curr_bed_type", crate::OptionValueKind::Enum, "Cool Plate"),
        ("default_bed_type", crate::OptionValueKind::String, ""),
        (
            "first_layer_print_sequence",
            crate::OptionValueKind::Ints,
            "0",
        ),
        (
            "first_layer_sequence_choice",
            crate::OptionValueKind::Enum,
            "Auto",
        ),
        (
            "other_layers_print_sequence",
            crate::OptionValueKind::Ints,
            "0",
        ),
        (
            "other_layers_print_sequence_nums",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "other_layers_sequence_choice",
            crate::OptionValueKind::Enum,
            "Auto",
        ),
        ("print_order", crate::OptionValueKind::Enum, "default"),
        ("print_sequence", crate::OptionValueKind::Enum, "by layer"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    for (key, kind, default_value) in [
        (
            "before_layer_change_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        ("bottom_shell_layers", crate::OptionValueKind::Int, "3"),
        ("bottom_shell_thickness", crate::OptionValueKind::Float, "0"),
        ("brim_ears", crate::OptionValueKind::Bool, "false"),
        (
            "brim_ears_detection_length",
            crate::OptionValueKind::Float,
            "1",
        ),
        ("brim_ears_max_angle", crate::OptionValueKind::Float, "125"),
        ("brim_flow_ratio", crate::OptionValueKind::Float, "1"),
        (
            "brim_use_efc_outline",
            crate::OptionValueKind::Bool,
            "false",
        ),
        ("combine_brims", crate::OptionValueKind::Bool, "false"),
        (
            "compatible_machine_expression_group",
            crate::OptionValueKind::Strings,
            "",
        ),
        ("compatible_printers", crate::OptionValueKind::Strings, ""),
        (
            "compatible_printers_condition",
            crate::OptionValueKind::String,
            "",
        ),
        ("compatible_prints", crate::OptionValueKind::Strings, ""),
        (
            "compatible_prints_condition",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "compatible_process_expression_group",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "different_settings_to_system",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "print_compatible_printers",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "upward_compatible_machine",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "bottom_solid_infill_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
        ("gap_fill_flow_ratio", crate::OptionValueKind::Float, "1"),
        ("gap_fill_target", crate::OptionValueKind::Enum, "nowhere"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
    for (key, kind, default_value) in [
        (
            "enable_overhang_bridge_fan",
            crate::OptionValueKind::Bools,
            "true",
        ),
        (
            "enable_overhang_speed",
            crate::OptionValueKind::Bool,
            "true",
        ),
        ("overhang_fan_speed", crate::OptionValueKind::Ints, "100"),
        (
            "overhang_fan_threshold",
            crate::OptionValueKind::Enums,
            "95%",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    for (key, kind, default_value) in [
        (
            "extra_perimeters_on_overhangs",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "min_width_top_surface",
            crate::OptionValueKind::FloatOrPercent,
            "300%",
        ),
        (
            "only_one_wall_first_layer",
            crate::OptionValueKind::Bool,
            "false",
        ),
        ("only_one_wall_top", crate::OptionValueKind::Bool, "false"),
        (
            "overhang_1_4_speed",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "overhang_2_4_speed",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "overhang_3_4_speed",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "overhang_4_4_speed",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        ("overhang_reverse", crate::OptionValueKind::Bool, "false"),
        (
            "overhang_reverse_internal_only",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "overhang_reverse_threshold",
            crate::OptionValueKind::FloatOrPercent,
            "50%",
        ),
        ("precise_outer_wall", crate::OptionValueKind::Bool, "true"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    for (key, kind, default_value) in [
        ("bridge_angle", crate::OptionValueKind::Float, "0"),
        ("bridge_density", crate::OptionValueKind::Percent, "100"),
        ("internal_bridge_angle", crate::OptionValueKind::Float, "0"),
        (
            "internal_bridge_density",
            crate::OptionValueKind::Percent,
            "100",
        ),
        (
            "top_solid_infill_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
        ("first_layer_flow_ratio", crate::OptionValueKind::Float, "1"),
        ("inner_wall_flow_ratio", crate::OptionValueKind::Float, "1"),
        ("outer_wall_flow_ratio", crate::OptionValueKind::Float, "1"),
        ("overhang_flow_ratio", crate::OptionValueKind::Float, "1"),
        (
            "set_other_flow_ratios",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "slowdown_for_curled_perimeters",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "sparse_infill_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
        ("support_flow_ratio", crate::OptionValueKind::Float, "1"),
        (
            "support_interface_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "internal_solid_infill_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
