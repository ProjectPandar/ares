#[test]
fn exposes_tree_support_branch_tip_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "tree_support_branch_angle",
            crate::OptionValueKind::Float,
            "40",
        ),
        (
            "tree_support_branch_angle_organic",
            crate::OptionValueKind::Float,
            "40",
        ),
        (
            "tree_support_angle_slow",
            crate::OptionValueKind::Float,
            "25",
        ),
        (
            "tree_support_branch_distance",
            crate::OptionValueKind::Float,
            "5",
        ),
        (
            "tree_support_branch_distance_organic",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "tree_support_top_rate",
            crate::OptionValueKind::Percent,
            "30",
        ),
        (
            "tree_support_auto_brim",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "tree_support_brim_width",
            crate::OptionValueKind::Float,
            "3",
        ),
        (
            "tree_support_tip_diameter",
            crate::OptionValueKind::Float,
            "0.8",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
