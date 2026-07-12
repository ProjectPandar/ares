#[test]
fn exposes_wipe_tower_placement_width_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "wipe_tower_x",
            crate::OptionValueKind::Floats,
            "15",
        ),
        (
            "wipe_tower_y",
            crate::OptionValueKind::Floats,
            "220",
        ),
        (
            "prime_tower_width",
            crate::OptionValueKind::Float,
            "60",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
