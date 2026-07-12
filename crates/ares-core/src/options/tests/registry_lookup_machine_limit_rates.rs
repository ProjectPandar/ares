#[test]
fn exposes_machine_limit_rate_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "machine_max_acceleration_extruding",
            crate::OptionValueKind::Floats,
            "1500,1250",
        ),
        (
            "machine_max_acceleration_retracting",
            crate::OptionValueKind::Floats,
            "1500,1250",
        ),
        (
            "machine_max_acceleration_travel",
            crate::OptionValueKind::Floats,
            "0,0",
        ),
        (
            "machine_max_jerk_e",
            crate::OptionValueKind::Floats,
            "2.5,2.5",
        ),
        (
            "machine_max_jerk_x",
            crate::OptionValueKind::Floats,
            "10,10",
        ),
        (
            "machine_max_jerk_y",
            crate::OptionValueKind::Floats,
            "10,10",
        ),
        (
            "machine_max_jerk_z",
            crate::OptionValueKind::Floats,
            "0.2,0.4",
        ),
        (
            "machine_max_junction_deviation",
            crate::OptionValueKind::Floats,
            "0.01",
        ),
        (
            "machine_min_extruding_rate",
            crate::OptionValueKind::Floats,
            "0,0",
        ),
        (
            "machine_min_travel_rate",
            crate::OptionValueKind::Floats,
            "0,0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
