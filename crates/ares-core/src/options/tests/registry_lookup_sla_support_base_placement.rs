#[test]
fn exposes_sla_support_base_placement_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_buildplate_only",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1699", "PrintConfig.cpp:7613-7618"][..],
        ),
        (
            "support_pillar_widening_factor",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1705", "PrintConfig.cpp:7620-7627"][..],
        ),
        (
            "support_base_diameter",
            crate::OptionValueKind::Float,
            "4",
            &["PrintConfig.hpp:1708", "PrintConfig.cpp:7629-7637"][..],
        ),
        (
            "support_base_height",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1711", "PrintConfig.cpp:7639-7646"][..],
        ),
        (
            "support_base_safety_distance",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1714", "PrintConfig.cpp:7648-7656"][..],
        ),
        (
            "support_critical_angle",
            crate::OptionValueKind::Float,
            "45",
            &["PrintConfig.hpp:1717", "PrintConfig.cpp:7658-7666"][..],
        ),
        (
            "support_max_bridge_length",
            crate::OptionValueKind::Float,
            "15",
            &["PrintConfig.hpp:1720", "PrintConfig.cpp:7668-7675"][..],
        ),
        (
            "support_max_pillar_link_distance",
            crate::OptionValueKind::Float,
            "10",
            &["PrintConfig.hpp:1723", "PrintConfig.cpp:7677-7684"][..],
        ),
        (
            "support_object_elevation",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1727", "PrintConfig.cpp:7686-7694"][..],
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
