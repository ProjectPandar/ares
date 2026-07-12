#[test]
fn exposes_timelapse_preheat_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "preheat_steps",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1567", "PrintConfig.cpp:5767-5774"][..],
        ),
        (
            "preheat_time",
            crate::OptionValueKind::Float,
            "30",
            &["PrintConfig.hpp:1566", "PrintConfig.cpp:5757-5765"][..],
        ),
        (
            "standby_temperature_delta",
            crate::OptionValueKind::Int,
            "-5",
            &["PrintConfig.hpp:1565", "PrintConfig.cpp:5745-5755"][..],
        ),
        (
            "timelapse_type",
            crate::OptionValueKind::Enum,
            "0",
            &[
                "PrintConfig.hpp:281-284",
                "PrintConfig.hpp:1615",
                "PrintConfig.cpp:431-435",
                "PrintConfig.cpp:5728-5743",
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
