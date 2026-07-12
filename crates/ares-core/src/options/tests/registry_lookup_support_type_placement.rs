#[test]
fn exposes_support_type_placement_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "support_angle",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "support_critical_regions_only",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "support_object_first_layer_gap",
            crate::OptionValueKind::Float,
            "0.2",
        ),
        (
            "support_object_xy_distance",
            crate::OptionValueKind::Float,
            "0.35",
        ),
        (
            "support_on_build_plate_only",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "support_remove_small_overhang",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "support_type",
            crate::OptionValueKind::Enum,
            "normal(auto)",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
