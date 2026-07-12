#[test]
fn exposes_nozzle_temperature_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "nozzle_temperature",
            crate::OptionValueKind::Ints,
            "200",
            &["PrintConfig.hpp:1568", "PrintConfig.cpp:6478-6485"][..],
        ),
        (
            "nozzle_temperature_range_high",
            crate::OptionValueKind::Ints,
            "240",
            &["PrintConfig.hpp:1572", "PrintConfig.cpp:6495-6501"][..],
        ),
        (
            "nozzle_temperature_range_low",
            crate::OptionValueKind::Ints,
            "190",
            &["PrintConfig.hpp:1571", "PrintConfig.cpp:6487-6493"][..],
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
