#[test]
fn exposes_support_ironing_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_ironing",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:997", "PrintConfig.cpp:6406-6412"][..],
        ),
        (
            "support_ironing_pattern",
            crate::OptionValueKind::Enum,
            "rectilinear",
            &[
                "PrintConfig.hpp:87-98",
                "PrintConfig.hpp:998",
                "PrintConfig.cpp:225-255",
                "PrintConfig.cpp:6414-6424",
            ][..],
        ),
        (
            "support_ironing_flow",
            crate::OptionValueKind::Percent,
            "10",
            &["PrintConfig.hpp:999", "PrintConfig.cpp:6426-6436"][..],
        ),
        (
            "support_ironing_spacing",
            crate::OptionValueKind::Float,
            "0.1",
            &["PrintConfig.hpp:1000", "PrintConfig.cpp:6438-6446"][..],
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
