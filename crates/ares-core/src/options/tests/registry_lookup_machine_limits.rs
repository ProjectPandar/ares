#[test]
fn exposes_machine_limit_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "machine_max_acceleration_e",
            crate::OptionValueKind::Floats,
            "5000,5000",
        ),
        (
            "machine_max_acceleration_x",
            crate::OptionValueKind::Floats,
            "1000,1000",
        ),
        (
            "machine_max_acceleration_y",
            crate::OptionValueKind::Floats,
            "1000,1000",
        ),
        (
            "machine_max_acceleration_z",
            crate::OptionValueKind::Floats,
            "500,200",
        ),
        (
            "machine_max_speed_e",
            crate::OptionValueKind::Floats,
            "120,120",
        ),
        (
            "machine_max_speed_x",
            crate::OptionValueKind::Floats,
            "500,200",
        ),
        (
            "machine_max_speed_y",
            crate::OptionValueKind::Floats,
            "500,200",
        ),
        (
            "machine_max_speed_z",
            crate::OptionValueKind::Floats,
            "12,12",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
