#[test]
fn exposes_sla_support_head_pillar_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "supports_enable",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "support_head_front_diameter",
            crate::OptionValueKind::Float,
            "0.4",
        ),
        (
            "support_head_penetration",
            crate::OptionValueKind::Float,
            "0.2",
        ),
        (
            "support_head_width",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "support_pillar_diameter",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "support_small_pillar_diameter_percent",
            crate::OptionValueKind::Percent,
            "50",
        ),
        (
            "support_max_bridges_on_pillar",
            crate::OptionValueKind::Int,
            "3",
        ),
        (
            "support_pillar_connection_mode",
            crate::OptionValueKind::Enum,
            "dynamic",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
