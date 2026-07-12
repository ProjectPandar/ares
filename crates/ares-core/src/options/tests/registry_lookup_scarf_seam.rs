#[test]
fn exposes_scarf_seam_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "scarf_angle_threshold",
            crate::OptionValueKind::Int,
            "155",
        ),
        (
            "scarf_joint_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "scarf_joint_speed",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
        ),
        (
            "scarf_overhang_threshold",
            crate::OptionValueKind::Percent,
            "40",
        ),
        (
            "seam_slope_conditional",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "seam_slope_entire_loop",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "seam_slope_inner_walls",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "seam_slope_min_length",
            crate::OptionValueKind::Float,
            "20",
        ),
        (
            "seam_slope_start_height",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "seam_slope_steps",
            crate::OptionValueKind::Int,
            "10",
        ),
        (
            "seam_slope_type",
            crate::OptionValueKind::Enum,
            "none",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
