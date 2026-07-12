#[test]
fn exposes_wall_transition_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "wall_distribution_count",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1024", "PrintConfig.cpp:7042-7049"][..],
        ),
        (
            "wall_transition_angle",
            crate::OptionValueKind::Float,
            "10",
            &["PrintConfig.hpp:1023", "PrintConfig.cpp:7029-7040"][..],
        ),
        (
            "wall_transition_filter_deviation",
            crate::OptionValueKind::Percent,
            "25",
            &["PrintConfig.hpp:1022", "PrintConfig.cpp:7014-7027"][..],
        ),
        (
            "wall_transition_length",
            crate::OptionValueKind::Percent,
            "100",
            &["PrintConfig.hpp:1021", "PrintConfig.cpp:7003-7012"][..],
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
