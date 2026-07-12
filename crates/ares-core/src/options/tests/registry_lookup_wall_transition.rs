#[test]
fn exposes_wall_transition_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "wall_distribution_count",
            crate::OptionValueKind::Int,
            "1",
        ),
        (
            "wall_transition_angle",
            crate::OptionValueKind::Float,
            "10",
        ),
        (
            "wall_transition_filter_deviation",
            crate::OptionValueKind::Percent,
            "25",
        ),
        (
            "wall_transition_length",
            crate::OptionValueKind::Percent,
            "100",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
