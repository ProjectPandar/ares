#[test]
fn exposes_support_spacing_speed_style_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_base_pattern_spacing",
            crate::OptionValueKind::Float,
            "2.5",
        ),
        (
            "support_expansion",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "support_speed",
            crate::OptionValueKind::Float,
            "80",
        ),
        (
            "support_style",
            crate::OptionValueKind::Enum,
            "default",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
