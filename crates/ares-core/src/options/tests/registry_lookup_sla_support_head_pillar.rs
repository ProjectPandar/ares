#[test]
fn exposes_sla_support_head_pillar_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "supports_enable",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1674; PrintConfig.cpp:7537-7542"][..],
        ),
        (
            "support_head_front_diameter",
            crate::OptionValueKind::Float,
            "0.4",
            &["PrintConfig.hpp:1677; PrintConfig.cpp:7544-7551"][..],
        ),
        (
            "support_head_penetration",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:1680; PrintConfig.cpp:7553-7560"][..],
        ),
        (
            "support_head_width",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1683; PrintConfig.cpp:7562-7570"][..],
        ),
        (
            "support_pillar_diameter",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1686; PrintConfig.cpp:7572-7580"][..],
        ),
        (
            "support_small_pillar_diameter_percent",
            crate::OptionValueKind::Percent,
            "50",
            &["PrintConfig.hpp:1690; PrintConfig.cpp:7582-7590"][..],
        ),
        (
            "support_max_bridges_on_pillar",
            crate::OptionValueKind::Int,
            "3",
            &["PrintConfig.hpp:1693; PrintConfig.cpp:7592-7600"][..],
        ),
        (
            "support_pillar_connection_mode",
            crate::OptionValueKind::Enum,
            "dynamic",
            &[
                "PrintConfig.hpp:265-269",
                "PrintConfig.hpp:1696",
                "PrintConfig.cpp:406-411",
                "PrintConfig.cpp:7600-7611",
            ][..],
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
