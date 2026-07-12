#[test]
fn exposes_support_interface_pattern_speed_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_base_pattern",
            crate::OptionValueKind::Enum,
            "default",
            &[
                "PrintConfig.hpp:172-177",
                "PrintConfig.hpp:969",
                "PrintConfig.cpp:312-320",
                "PrintConfig.cpp:6133-6156",
            ][..],
        ),
        (
            "support_bottom_interface_spacing",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1019", "PrintConfig.cpp:6114-6122"][..],
        ),
        (
            "support_interface_pattern",
            crate::OptionValueKind::Enum,
            "auto",
            &[
                "PrintConfig.hpp:190-192",
                "PrintConfig.hpp:970",
                "PrintConfig.cpp:333-340",
                "PrintConfig.cpp:6158-6176",
            ][..],
        ),
        (
            "support_interface_speed",
            crate::OptionValueKind::Float,
            "80",
            &["PrintConfig.hpp:968", "PrintConfig.cpp:6124-6131"][..],
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
