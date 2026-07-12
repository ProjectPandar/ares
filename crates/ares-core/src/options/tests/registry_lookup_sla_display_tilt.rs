#[test]
fn exposes_sla_display_tilt_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "area_fill",
            crate::OptionValueKind::Float,
            "50",
        ),
        (
            "display_height",
            crate::OptionValueKind::Float,
            "68",
        ),
        (
            "display_mirror_x",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "display_mirror_y",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "display_orientation",
            crate::OptionValueKind::Enum,
            "portrait",
        ),
        (
            "display_pixels_x",
            crate::OptionValueKind::Int,
            "2560",
        ),
        (
            "display_pixels_y",
            crate::OptionValueKind::Int,
            "1440",
        ),
        (
            "display_width",
            crate::OptionValueKind::Float,
            "120",
        ),
        (
            "fast_tilt_time",
            crate::OptionValueKind::Float,
            "5",
        ),
        (
            "slow_tilt_time",
            crate::OptionValueKind::Float,
            "8",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
