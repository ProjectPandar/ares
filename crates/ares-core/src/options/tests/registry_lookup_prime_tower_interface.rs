#[test]
fn exposes_prime_tower_interface_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_tower_interface_cooldown_during_tower",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1587", "PrintConfig.cpp:6833-6837"][..],
        ),
        (
            "enable_tower_interface_features",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1586", "PrintConfig.cpp:6827-6831"][..],
        ),
        (
            "prime_tower_flat_ironing",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1585", "PrintConfig.cpp:6823-6825"][..],
        ),
        (
            "prime_tower_infill_gap",
            crate::OptionValueKind::Percent,
            "150",
            &["PrintConfig.hpp:1583", "PrintConfig.cpp:6839-6845"][..],
        ),
        (
            "prime_tower_skip_points",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1584", "PrintConfig.cpp:6817-6821"][..],
        ),
        (
            "wiping_volumes_extruders",
            crate::OptionValueKind::Floats,
            "70,70,70,70,70,70,70,70,70,70",
            &["PrintConfig.hpp:1602", "PrintConfig.cpp:6810-6815"][..],
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
