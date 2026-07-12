#[test]
fn exposes_wipe_prime_tower_base_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "wipe",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1569", "PrintConfig.cpp:6628-6633"][..],
        ),
        (
            "wipe_distance",
            crate::OptionValueKind::Floats,
            "1",
            &["PrintConfig.hpp:1573", "PrintConfig.cpp:6635-6644"][..],
        ),
        (
            "enable_prime_tower",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1574", "PrintConfig.cpp:6646-6651"][..],
        ),
        (
            "prime_tower_enable_framework",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1575", "PrintConfig.cpp:6653-6657"][..],
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
