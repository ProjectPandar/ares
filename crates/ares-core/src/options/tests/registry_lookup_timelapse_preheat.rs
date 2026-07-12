#[test]
fn exposes_timelapse_preheat_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "preheat_steps",
            crate::OptionValueKind::Int,
            "1",
        ),
        (
            "preheat_time",
            crate::OptionValueKind::Float,
            "30",
        ),
        (
            "standby_temperature_delta",
            crate::OptionValueKind::Int,
            "-5",
        ),
        (
            "timelapse_type",
            crate::OptionValueKind::Enum,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
