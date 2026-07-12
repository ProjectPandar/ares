#[test]
fn exposes_support_threshold_layer_height_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "independent_support_layer_height",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1618", "PrintConfig.cpp:6232-6238"][..],
        ),
        (
            "support_threshold_angle",
            crate::OptionValueKind::Int,
            "30",
            &["PrintConfig.hpp:993", "PrintConfig.cpp:6240-6251"][..],
        ),
        (
            "support_threshold_overlap",
            crate::OptionValueKind::FloatOrPercent,
            "50%",
            &["PrintConfig.hpp:994", "PrintConfig.cpp:6253-6262"][..],
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
