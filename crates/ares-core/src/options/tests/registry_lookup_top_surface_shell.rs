#[test]
fn exposes_top_surface_shell_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "top_shell_layers",
            crate::OptionValueKind::Int,
            "4",
        ),
        (
            "top_shell_thickness",
            crate::OptionValueKind::Float,
            "0.6",
        ),
        (
            "top_surface_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "top_surface_speed",
            crate::OptionValueKind::Float,
            "100",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
