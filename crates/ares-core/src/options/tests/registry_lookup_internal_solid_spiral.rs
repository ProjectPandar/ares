#[test]
fn exposes_internal_solid_spiral_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "internal_solid_infill_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "internal_solid_infill_speed",
            crate::OptionValueKind::Float,
            "100",
        ),
        (
            "spiral_finishing_flow_ratio",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "spiral_mode",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "spiral_mode_max_xy_smoothing",
            crate::OptionValueKind::FloatOrPercent,
            "200%",
        ),
        (
            "spiral_mode_smooth",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "spiral_starting_flow_ratio",
            crate::OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
