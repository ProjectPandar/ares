#[test]
fn exposes_travel_speed_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "travel_speed",
            crate::OptionValueKind::Float,
            "120",
            &["PrintConfig.hpp:1396", "PrintConfig.cpp:6610-6616"][..],
        ),
        (
            "travel_speed_z",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1397", "PrintConfig.cpp:6618-6626"][..],
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
