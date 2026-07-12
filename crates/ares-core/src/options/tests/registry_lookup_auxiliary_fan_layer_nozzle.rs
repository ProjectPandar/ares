#[test]
fn exposes_auxiliary_fan_layer_nozzle_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "additional_cooling_fan_speed",
            crate::OptionValueKind::Ints,
            "0",
            &["PrintConfig.hpp:1475", "PrintConfig.cpp:4660-4668"][..],
        ),
        (
            "additional_fan_full_speed_layer",
            crate::OptionValueKind::Ints,
            "0",
            &["PrintConfig.hpp:1477", "PrintConfig.cpp:4679-4686"][..],
        ),
        (
            "close_additional_fan_first_x_layers",
            crate::OptionValueKind::Ints,
            "1",
            &["PrintConfig.hpp:1476", "PrintConfig.cpp:4670-4677"][..],
        ),
        (
            "fan_min_speed",
            crate::OptionValueKind::Floats,
            "20",
            &["PrintConfig.hpp:1537", "PrintConfig.cpp:4651-4658"][..],
        ),
        (
            "first_x_layer_fan_speed",
            crate::OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1478", "PrintConfig.cpp:4688-4695"][..],
        ),
        (
            "min_layer_height",
            crate::OptionValueKind::Floats,
            "0.07",
            &["PrintConfig.hpp:1538", "PrintConfig.cpp:4697-4704"][..],
        ),
        (
            "nozzle_diameter",
            crate::OptionValueKind::Floats,
            "0.4",
            &["PrintConfig.hpp:1543", "PrintConfig.cpp:4715-4721"][..],
        ),
        (
            "slow_down_min_speed",
            crate::OptionValueKind::Floats,
            "10",
            &["PrintConfig.hpp:1542", "PrintConfig.cpp:4706-4713"][..],
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
