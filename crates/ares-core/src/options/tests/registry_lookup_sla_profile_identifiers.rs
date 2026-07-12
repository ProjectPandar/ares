#[test]
fn exposes_sla_profile_identifiers_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "default_sla_material_profile",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:7513-7517"][..],
        ),
        (
            "default_sla_print_profile",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:7525-7529"][..],
        ),
        (
            "material_vendor",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:7507-7511"][..],
        ),
        (
            "sla_material_settings_id",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:7519-7523"][..],
        ),
        (
            "sla_print_settings_id",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:7531-7535"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
