#[test]
fn exposes_prime_tower_interface_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enable_tower_interface_cooldown_during_tower",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "enable_tower_interface_features",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "prime_tower_flat_ironing",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "prime_tower_infill_gap",
            crate::OptionValueKind::Percent,
            "150",
        ),
        (
            "prime_tower_skip_points",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "wiping_volumes_extruders",
            crate::OptionValueKind::Floats,
            "70,70,70,70,70,70,70,70,70,70",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
