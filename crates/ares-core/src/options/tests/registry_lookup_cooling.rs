#[test]
fn exposes_cooling_slowdown_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "dont_slow_down_outer_wall",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1520", "PrintConfig.cpp:2340-2347"][..],
        ),
        (
            "full_fan_speed_layer",
            crate::OptionValueKind::Ints,
            "0",
            &["PrintConfig.hpp:1534", "PrintConfig.cpp:3325-3335"][..],
        ),
        (
            "internal_bridge_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
            &["PrintConfig.hpp:1629", "PrintConfig.cpp:3350-3359"][..],
        ),
        (
            "ironing_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
            &["PrintConfig.hpp:1630", "PrintConfig.cpp:3361-3370"][..],
        ),
        (
            "nozzle_temperature_initial_layer",
            crate::OptionValueKind::Ints,
            "200",
            &["PrintConfig.hpp:1533", "PrintConfig.cpp:3316-3323"][..],
        ),
        (
            "reduce_fan_stop_start_freq",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1519", "PrintConfig.cpp:2334-2338"][..],
        ),
        (
            "support_material_interface_fan_speed",
            crate::OptionValueKind::Ints,
            "-1",
            &["PrintConfig.hpp:1628", "PrintConfig.cpp:3337-3347"][..],
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
