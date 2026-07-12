#[test]
fn exposes_wall_maximum_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "wall_maximum_deviation",
            crate::OptionValueKind::Float,
            "0.025",
            &["PrintConfig.hpp:1031", "PrintConfig.cpp:7087-7097"][..],
        ),
        (
            "wall_maximum_resolution",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1030", "PrintConfig.cpp:7076-7085"][..],
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
