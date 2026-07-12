#[test]
fn exposes_sla_support_points_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_points_density_relative",
            crate::OptionValueKind::Int,
            "100",
        ),
        (
            "support_points_minimal_distance",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
