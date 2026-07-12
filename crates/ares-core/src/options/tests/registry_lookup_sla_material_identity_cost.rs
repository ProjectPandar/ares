#[test]
fn exposes_sla_material_identity_cost_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "bottle_cost",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "bottle_volume",
            crate::OptionValueKind::Float,
            "1000",
        ),
        (
            "bottle_weight",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "material_colour",
            crate::OptionValueKind::String,
            "#29B2B2",
        ),
        (
            "material_density",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "material_type",
            crate::OptionValueKind::String,
            "Tough",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
