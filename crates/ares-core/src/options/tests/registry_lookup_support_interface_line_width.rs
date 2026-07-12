#[test]
fn exposes_support_interface_line_width_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_interface_not_for_body",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:961", "PrintConfig.cpp:6036-6041"][..],
        ),
        (
            "support_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:960", "PrintConfig.cpp:6043-6053"][..],
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
