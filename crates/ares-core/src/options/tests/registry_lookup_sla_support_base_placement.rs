#[test]
fn exposes_sla_support_base_placement_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_buildplate_only",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "support_pillar_widening_factor",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "support_base_diameter",
            crate::OptionValueKind::Float,
            "4",
        ),
        (
            "support_base_height",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "support_base_safety_distance",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "support_critical_angle",
            crate::OptionValueKind::Float,
            "45",
        ),
        (
            "support_max_bridge_length",
            crate::OptionValueKind::Float,
            "15",
        ),
        (
            "support_max_pillar_link_distance",
            crate::OptionValueKind::Float,
            "10",
        ),
        (
            "support_object_elevation",
            crate::OptionValueKind::Float,
            "5",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
