#[test]
fn exposes_sla_material_identity_cost_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "bottle_cost",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1811", "PrintConfig.cpp:7418-7423"][..],
        ),
        (
            "bottle_volume",
            crate::OptionValueKind::Float,
            "1000",
            &["PrintConfig.hpp:1812", "PrintConfig.cpp:7397-7402"][..],
        ),
        (
            "bottle_weight",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1813", "PrintConfig.cpp:7404-7409"][..],
        ),
        (
            "material_colour",
            crate::OptionValueKind::String,
            "#29B2B2",
            &["PrintConfig.cpp:7372-7376"][..],
        ),
        (
            "material_density",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1814", "PrintConfig.cpp:7411-7416"][..],
        ),
        (
            "material_type",
            crate::OptionValueKind::String,
            "Tough",
            &["PrintConfig.cpp:7378-7388"][..],
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
