#[test]
fn exposes_wipe_prime_tower_base_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "wipe",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "wipe_distance",
            crate::OptionValueKind::Floats,
            "1",
        ),
        (
            "enable_prime_tower",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "prime_tower_enable_framework",
            crate::OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
