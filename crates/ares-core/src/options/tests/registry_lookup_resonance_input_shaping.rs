#[test]
fn exposes_resonance_input_shaping_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "input_shaping_damp_x",
            crate::OptionValueKind::Float,
            "0.1",
        ),
        (
            "input_shaping_damp_y",
            crate::OptionValueKind::Float,
            "0.1",
        ),
        (
            "input_shaping_emit",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "input_shaping_freq_x",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "input_shaping_freq_y",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "input_shaping_type",
            crate::OptionValueKind::Enum,
            "Default",
        ),
        (
            "max_resonance_avoidance_speed",
            crate::OptionValueKind::Float,
            "120",
        ),
        (
            "min_resonance_avoidance_speed",
            crate::OptionValueKind::Float,
            "70",
        ),
        (
            "resonance_avoidance",
            crate::OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
