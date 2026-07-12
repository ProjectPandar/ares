#[test]
fn exposes_bead_width_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "initial_layer_min_bead_width",
            crate::OptionValueKind::Percent,
            "85",
            &["PrintConfig.hpp:1026", "PrintConfig.cpp:7099-7107"][..],
        ),
        (
            "min_bead_width",
            crate::OptionValueKind::Percent,
            "85",
            &["PrintConfig.hpp:1027", "PrintConfig.cpp:7109-7119"][..],
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
