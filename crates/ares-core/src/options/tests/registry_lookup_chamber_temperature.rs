#[test]
fn exposes_chamber_temperature_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "activate_chamber_temp_control",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1636", "PrintConfig.cpp:6448-6455"][..],
        ),
        (
            "chamber_temperature",
            crate::OptionValueKind::Ints,
            "0",
            &["PrintConfig.hpp:1637", "PrintConfig.cpp:6457-6476"][..],
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
