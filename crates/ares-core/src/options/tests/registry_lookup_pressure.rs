#[test]
fn exposes_pressure_advance_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "adaptive_pressure_advance",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "adaptive_pressure_advance_bridges",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "adaptive_pressure_advance_model",
            crate::OptionValueKind::Strings,
            "0,0,0\n0,0,0",
        ),
        (
            "adaptive_pressure_advance_overhangs",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "enable_pressure_advance",
            crate::OptionValueKind::Bools,
            "false",
        ),
        ("pressure_advance", crate::OptionValueKind::Floats, "0.02"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
