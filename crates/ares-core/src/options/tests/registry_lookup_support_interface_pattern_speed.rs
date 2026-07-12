#[test]
fn exposes_support_interface_pattern_speed_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_base_pattern",
            crate::OptionValueKind::Enum,
            "default",
        ),
        (
            "support_bottom_interface_spacing",
            crate::OptionValueKind::Float,
            "0.5",
        ),
        (
            "support_interface_pattern",
            crate::OptionValueKind::Enum,
            "auto",
        ),
        (
            "support_interface_speed",
            crate::OptionValueKind::Float,
            "80",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
