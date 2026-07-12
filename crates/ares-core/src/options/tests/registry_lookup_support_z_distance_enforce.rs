#[test]
fn exposes_support_z_distance_enforce_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enforce_support_layers",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "support_bottom_z_distance",
            crate::OptionValueKind::Float,
            "0.2",
        ),
        (
            "support_top_z_distance",
            crate::OptionValueKind::Float,
            "0.2",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
