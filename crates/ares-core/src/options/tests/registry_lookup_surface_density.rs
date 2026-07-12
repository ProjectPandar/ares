#[test]
fn exposes_surface_density_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "bottom_surface_density",
            crate::OptionValueKind::Percent,
            "100",
        ),
        (
            "top_surface_density",
            crate::OptionValueKind::Percent,
            "100",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
