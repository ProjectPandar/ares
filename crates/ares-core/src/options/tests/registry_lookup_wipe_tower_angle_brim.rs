#[test]
fn exposes_wipe_tower_angle_brim_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "wipe_tower_rotation_angle",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1581", "PrintConfig.cpp:6718-6723"][..],
        ),
        (
            "prime_tower_brim_width",
            crate::OptionValueKind::Float,
            "3",
            &["PrintConfig.hpp:1582", "PrintConfig.cpp:6725-6734"][..],
        ),
        (
            "wipe_tower_cone_angle",
            crate::OptionValueKind::Float,
            "30",
            &["PrintConfig.hpp:1594", "PrintConfig.cpp:6736-6744"][..],
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
