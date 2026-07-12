#[test]
fn exposes_tree_support_branch_tip_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "tree_support_branch_angle",
            crate::OptionValueKind::Float,
            "40",
            &["PrintConfig.hpp:1011", "PrintConfig.cpp:6264-6273"][..],
        ),
        (
            "tree_support_branch_angle_organic",
            crate::OptionValueKind::Float,
            "40",
            &["PrintConfig.hpp:1037", "PrintConfig.cpp:6275-6284"][..],
        ),
        (
            "tree_support_angle_slow",
            crate::OptionValueKind::Float,
            "25",
            &["PrintConfig.hpp:1013", "PrintConfig.cpp:6286-6296"][..],
        ),
        (
            "tree_support_branch_distance",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1008", "PrintConfig.cpp:6298-6306"][..],
        ),
        (
            "tree_support_branch_distance_organic",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1034", "PrintConfig.cpp:6308-6316"][..],
        ),
        (
            "tree_support_top_rate",
            crate::OptionValueKind::Percent,
            "30",
            &["PrintConfig.hpp:1035", "PrintConfig.cpp:6318-6330"][..],
        ),
        (
            "tree_support_auto_brim",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1015", "PrintConfig.cpp:6332-6336"][..],
        ),
        (
            "tree_support_brim_width",
            crate::OptionValueKind::Float,
            "3",
            &["PrintConfig.hpp:1016", "PrintConfig.cpp:6338-6343"][..],
        ),
        (
            "tree_support_tip_diameter",
            crate::OptionValueKind::Float,
            "0.8",
            &["PrintConfig.hpp:1009", "PrintConfig.cpp:6345-6354"][..],
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
