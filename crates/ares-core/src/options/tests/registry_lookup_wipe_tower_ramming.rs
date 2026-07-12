#[test]
fn exposes_wipe_tower_ramming_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_filament_ramming",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1459", "PrintConfig.cpp:5838-5842"][..],
        ),
        (
            "purge_in_prime_tower",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1458", "PrintConfig.cpp:5832-5836"][..],
        ),
        (
            "tool_change_on_wipe_tower",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1460", "PrintConfig.cpp:5844-5852"][..],
        ),
        (
            "wipe_tower_no_sparse_layers",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1391", "PrintConfig.cpp:5855-5861"][..],
        ),
        (
            "wipe_tower_type",
            crate::OptionValueKind::Enum,
            "type2",
            &[
                "PrintConfig.hpp:74-77",
                "PrintConfig.hpp:1457",
                "PrintConfig.cpp:212-216",
                "PrintConfig.cpp:5821-5830",
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
