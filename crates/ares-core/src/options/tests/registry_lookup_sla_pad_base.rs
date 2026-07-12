#[test]
fn exposes_sla_pad_base_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "pad_brim_size",
            crate::OptionValueKind::Float,
            "1.6",
            &["PrintConfig.hpp:1745", "PrintConfig.cpp:7739-7747"][..],
        ),
        (
            "pad_enable",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1736", "PrintConfig.cpp:7712-7717"][..],
        ),
        (
            "pad_max_merge_distance",
            crate::OptionValueKind::Float,
            "50",
            &["PrintConfig.hpp:1749", "PrintConfig.cpp:7749-7756"][..],
        ),
        (
            "pad_wall_height",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1742", "PrintConfig.cpp:7729-7737"][..],
        ),
        (
            "pad_wall_slope",
            crate::OptionValueKind::Float,
            "90",
            &["PrintConfig.hpp:1755", "PrintConfig.cpp:7758-7766"][..],
        ),
        (
            "pad_wall_thickness",
            crate::OptionValueKind::Float,
            "2",
            &["PrintConfig.hpp:1739", "PrintConfig.cpp:7719-7727"][..],
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
