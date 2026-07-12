#[test]
fn exposes_wipe_tower_placement_width_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "wipe_tower_x",
            crate::OptionValueKind::Floats,
            "15",
            &["PrintConfig.hpp:1577", "PrintConfig.cpp:6694-6700"][..],
        ),
        (
            "wipe_tower_y",
            crate::OptionValueKind::Floats,
            "220",
            &["PrintConfig.hpp:1578", "PrintConfig.cpp:6702-6708"][..],
        ),
        (
            "prime_tower_width",
            crate::OptionValueKind::Float,
            "60",
            &["PrintConfig.hpp:1579", "PrintConfig.cpp:6710-6716"][..],
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
