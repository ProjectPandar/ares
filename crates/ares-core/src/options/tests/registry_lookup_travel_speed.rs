#[test]
fn exposes_travel_speed_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "travel_speed",
            crate::OptionValueKind::Float,
            "120",
        ),
        (
            "travel_speed_z",
            crate::OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
