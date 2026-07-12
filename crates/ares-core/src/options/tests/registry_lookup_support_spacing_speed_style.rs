#[test]
fn exposes_support_spacing_speed_style_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_base_pattern_spacing",
            crate::OptionValueKind::Float,
            "2.5",
            &["PrintConfig.hpp:972", "PrintConfig.cpp:6178-6185"][..],
        ),
        (
            "support_expansion",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:973", "PrintConfig.cpp:6187-6193"][..],
        ),
        (
            "support_speed",
            crate::OptionValueKind::Float,
            "80",
            &["PrintConfig.hpp:974", "PrintConfig.cpp:6195-6202"][..],
        ),
        (
            "support_style",
            crate::OptionValueKind::Enum,
            "default",
            &[
                "PrintConfig.hpp:179-181",
                "PrintConfig.hpp:975",
                "PrintConfig.cpp:322-331",
                "PrintConfig.cpp:6204-6230",
            ][..],
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
