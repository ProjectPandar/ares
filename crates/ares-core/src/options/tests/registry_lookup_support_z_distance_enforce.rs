#[test]
fn exposes_support_z_distance_enforce_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enforce_support_layers",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:958", "PrintConfig.cpp:6013-6025"][..],
        ),
        (
            "support_bottom_z_distance",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:957", "PrintConfig.cpp:6002-6011"][..],
        ),
        (
            "support_top_z_distance",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:956", "PrintConfig.cpp:5981-6000"][..],
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
