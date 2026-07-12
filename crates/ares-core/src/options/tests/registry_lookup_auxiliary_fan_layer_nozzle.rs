#[test]
fn exposes_auxiliary_fan_layer_nozzle_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "additional_cooling_fan_speed",
            crate::OptionValueKind::Ints,
            "0",
        ),
        (
            "additional_fan_full_speed_layer",
            crate::OptionValueKind::Ints,
            "0",
        ),
        (
            "close_additional_fan_first_x_layers",
            crate::OptionValueKind::Ints,
            "1",
        ),
        (
            "fan_min_speed",
            crate::OptionValueKind::Floats,
            "20",
        ),
        (
            "first_x_layer_fan_speed",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "min_layer_height",
            crate::OptionValueKind::Floats,
            "0.07",
        ),
        (
            "nozzle_diameter",
            crate::OptionValueKind::Floats,
            "0.4",
        ),
        (
            "slow_down_min_speed",
            crate::OptionValueKind::Floats,
            "10",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
