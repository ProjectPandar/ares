#[test]
fn exposes_sla_support_points_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_points_density_relative",
            crate::OptionValueKind::Int,
            "100",
            &["PrintConfig.hpp:1730", "PrintConfig.cpp:7696-7702"][..],
        ),
        (
            "support_points_minimal_distance",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1731", "PrintConfig.cpp:7704-7710"][..],
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
