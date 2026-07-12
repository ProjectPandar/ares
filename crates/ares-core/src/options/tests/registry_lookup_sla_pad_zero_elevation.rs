#[test]
fn exposes_sla_pad_zero_elevation_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "pad_around_object",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1766", "PrintConfig.cpp:7768-7773"][..],
        ),
        (
            "pad_around_object_everywhere",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1768", "PrintConfig.cpp:7775-7780"][..],
        ),
        (
            "pad_object_connector_penetration",
            crate::OptionValueKind::Float,
            "0.3",
            &["PrintConfig.hpp:1780", "PrintConfig.cpp:7810-7817"][..],
        ),
        (
            "pad_object_connector_stride",
            crate::OptionValueKind::Float,
            "10",
            &["PrintConfig.hpp:1774", "PrintConfig.cpp:7792-7799"][..],
        ),
        (
            "pad_object_connector_width",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1777", "PrintConfig.cpp:7801-7808"][..],
        ),
        (
            "pad_object_gap",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1771", "PrintConfig.cpp:7782-7790"][..],
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
