#[test]
fn exposes_tree_support_diameter_wall_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "tree_support_branch_diameter",
            crate::OptionValueKind::Float,
            "5",
        ),
        (
            "tree_support_branch_diameter_angle",
            crate::OptionValueKind::Float,
            "5",
        ),
        (
            "tree_support_branch_diameter_organic",
            crate::OptionValueKind::Float,
            "2",
        ),
        (
            "tree_support_wall_count",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "tree_support_with_infill",
            crate::OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
