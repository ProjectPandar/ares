#[test]
fn exposes_wipe_tower_ramming_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enable_filament_ramming",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "purge_in_prime_tower",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "tool_change_on_wipe_tower",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "wipe_tower_no_sparse_layers",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "wipe_tower_type",
            crate::OptionValueKind::Enum,
            "type2",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
