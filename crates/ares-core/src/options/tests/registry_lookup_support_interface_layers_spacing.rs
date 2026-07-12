#[test]
fn exposes_support_interface_layers_spacing_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_interface_bottom_layers",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "support_interface_filament",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "support_interface_loop_pattern",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "support_interface_spacing",
            crate::OptionValueKind::Float,
            "0.5",
        ),
        (
            "support_interface_top_layers",
            crate::OptionValueKind::Int,
            "3",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
