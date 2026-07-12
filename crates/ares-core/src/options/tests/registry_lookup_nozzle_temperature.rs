#[test]
fn exposes_nozzle_temperature_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "nozzle_temperature",
            crate::OptionValueKind::Ints,
            "200",
        ),
        (
            "nozzle_temperature_range_high",
            crate::OptionValueKind::Ints,
            "240",
        ),
        (
            "nozzle_temperature_range_low",
            crate::OptionValueKind::Ints,
            "190",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
