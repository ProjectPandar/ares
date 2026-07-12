#[test]
fn exposes_wall_maximum_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "wall_maximum_deviation",
            crate::OptionValueKind::Float,
            "0.025",
        ),
        (
            "wall_maximum_resolution",
            crate::OptionValueKind::Float,
            "0.5",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
