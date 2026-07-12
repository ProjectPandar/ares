#[test]
fn exposes_support_ironing_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_ironing",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "support_ironing_pattern",
            crate::OptionValueKind::Enum,
            "rectilinear",
        ),
        (
            "support_ironing_flow",
            crate::OptionValueKind::Percent,
            "10",
        ),
        (
            "support_ironing_spacing",
            crate::OptionValueKind::Float,
            "0.1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
