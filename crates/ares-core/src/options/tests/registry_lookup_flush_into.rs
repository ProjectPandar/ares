#[test]
fn exposes_flush_into_option_definition_lookup() {
    for (key, default_value, source_fragments) in [
        (
            "flush_into_infill",
            "false",
            &["PrintConfig.hpp:1005", "PrintConfig.cpp:6847-6854"][..],
        ),
        (
            "flush_into_objects",
            "false",
            &["PrintConfig.hpp:1003", "PrintConfig.cpp:6864-6870"][..],
        ),
        (
            "flush_into_support",
            "true",
            &["PrintConfig.hpp:1006", "PrintConfig.cpp:6856-6862"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Bool);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
