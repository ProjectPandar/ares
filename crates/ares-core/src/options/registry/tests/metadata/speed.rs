use super::super::super::{OptionValueKind, option_definition};

#[test]
fn speed_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("accel_to_decel_enable", OptionValueKind::Bool, "true"),
        ("accel_to_decel_factor", OptionValueKind::Percent, "50"),
        (
            "bridge_acceleration",
            OptionValueKind::FloatOrPercent,
            "50%",
        ),
        ("default_jerk", OptionValueKind::Float, "0"),
        ("default_junction_deviation", OptionValueKind::Float, "0"),
        ("infill_jerk", OptionValueKind::Float, "9"),
        ("enable_overhang_speed", OptionValueKind::Bool, "true"),
        ("initial_layer_acceleration", OptionValueKind::Float, "300"),
        ("initial_layer_infill_speed", OptionValueKind::Float, "60"),
        ("initial_layer_jerk", OptionValueKind::Float, "9"),
        (
            "initial_layer_line_width",
            OptionValueKind::FloatOrPercent,
            "0",
        ),
        ("initial_layer_print_height", OptionValueKind::Float, "0.2"),
        ("initial_layer_speed", OptionValueKind::Float, "30"),
        (
            "initial_layer_travel_acceleration",
            OptionValueKind::FloatOrPercent,
            "100%",
        ),
        (
            "initial_layer_travel_jerk",
            OptionValueKind::FloatOrPercent,
            "100%",
        ),
        (
            "initial_layer_travel_speed",
            OptionValueKind::FloatOrPercent,
            "100%",
        ),
        ("inner_wall_acceleration", OptionValueKind::Float, "10000"),
        ("inner_wall_jerk", OptionValueKind::Float, "9"),
        (
            "internal_solid_infill_acceleration",
            OptionValueKind::FloatOrPercent,
            "100%",
        ),
        ("outer_wall_acceleration", OptionValueKind::Float, "500"),
        ("outer_wall_jerk", OptionValueKind::Float, "9"),
        (
            "slowdown_for_curled_perimeters",
            OptionValueKind::Bool,
            "false",
        ),
        ("slow_down_layers", OptionValueKind::Int, "0"),
        (
            "sparse_infill_acceleration",
            OptionValueKind::FloatOrPercent,
            "100%",
        ),
        (
            "small_perimeter_speed",
            OptionValueKind::FloatOrPercent,
            "50%",
        ),
        ("overhang_1_4_speed", OptionValueKind::FloatOrPercent, "0"),
        ("overhang_2_4_speed", OptionValueKind::FloatOrPercent, "0"),
        ("overhang_3_4_speed", OptionValueKind::FloatOrPercent, "0"),
        ("overhang_4_4_speed", OptionValueKind::FloatOrPercent, "0"),
        ("top_surface_acceleration", OptionValueKind::Float, "500"),
        ("top_surface_jerk", OptionValueKind::Float, "9"),
        ("travel_acceleration", OptionValueKind::Float, "10000"),
        ("travel_jerk", OptionValueKind::Float, "12"),
        ("pellet_flow_coefficient", OptionValueKind::Floats, "0.4157"),
        (
            "volumetric_speed_coefficients",
            OptionValueKind::Strings,
            "",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
