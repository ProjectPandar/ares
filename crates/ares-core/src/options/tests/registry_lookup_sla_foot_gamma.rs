#[test]
fn exposes_sla_foot_gamma_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "elefant_foot_min_width",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:1843", "PrintConfig.cpp:7351-7358"][..],
        ),
        (
            "gamma_correction",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1844", "PrintConfig.cpp:7360-7367"][..],
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
