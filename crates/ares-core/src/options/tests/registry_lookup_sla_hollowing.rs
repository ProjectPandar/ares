#[test]
fn exposes_sla_hollowing_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "hollowing_closing_distance",
            crate::OptionValueKind::Float,
            "2",
        ),
        (
            "hollowing_enable",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "hollowing_min_thickness",
            crate::OptionValueKind::Float,
            "3",
        ),
        (
            "hollowing_quality",
            crate::OptionValueKind::Float,
            "0.5",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
