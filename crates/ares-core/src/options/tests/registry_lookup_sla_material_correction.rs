#[test]
fn exposes_sla_material_correction_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "material_correction",
            crate::OptionValueKind::Floats,
            "1",
            &["PrintConfig.hpp:1817", "PrintConfig.cpp:7479-7484"][..],
        ),
        (
            "material_correction_x",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1818", "PrintConfig.cpp:7486-7491"][..],
        ),
        (
            "material_correction_y",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1819", "PrintConfig.cpp:7493-7498"][..],
        ),
        (
            "material_correction_z",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1820", "PrintConfig.cpp:7500-7505"][..],
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
