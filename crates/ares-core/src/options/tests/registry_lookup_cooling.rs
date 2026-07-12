#[test]
fn exposes_cooling_slowdown_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "dont_slow_down_outer_wall",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "full_fan_speed_layer",
            crate::OptionValueKind::Ints,
            "0",
        ),
        (
            "internal_bridge_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
        ),
        (
            "ironing_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
        ),
        (
            "nozzle_temperature_initial_layer",
            crate::OptionValueKind::Ints,
            "200",
        ),
        (
            "reduce_fan_stop_start_freq",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "support_material_interface_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
