#[test]
fn exposes_sla_profile_identifiers_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "default_sla_material_profile",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "default_sla_print_profile",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "material_vendor",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "sla_material_settings_id",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "sla_print_settings_id",
            crate::OptionValueKind::String,
            "",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
