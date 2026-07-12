#[test]
fn exposes_surface_density_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "bottom_surface_density",
            crate::OptionValueKind::Percent,
            "100",
            &["PrintConfig.hpp:1089", "PrintConfig.cpp:6598-6607"][..],
        ),
        (
            "top_surface_density",
            crate::OptionValueKind::Percent,
            "100",
            &["PrintConfig.hpp:1088", "PrintConfig.cpp:6586-6596"][..],
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
