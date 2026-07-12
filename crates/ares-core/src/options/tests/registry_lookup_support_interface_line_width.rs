#[test]
fn exposes_support_interface_line_width_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_interface_not_for_body",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "support_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
