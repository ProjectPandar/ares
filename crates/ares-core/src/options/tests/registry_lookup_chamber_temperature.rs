#[test]
fn exposes_chamber_temperature_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "activate_chamber_temp_control",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "chamber_temperature",
            crate::OptionValueKind::Ints,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
