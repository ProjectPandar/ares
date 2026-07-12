#[test]
fn exposes_tree_support_diameter_wall_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "tree_support_branch_diameter",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1010", "PrintConfig.cpp:6356-6364"][..],
        ),
        (
            "tree_support_branch_diameter_angle",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1012", "PrintConfig.cpp:6366-6378"][..],
        ),
        (
            "tree_support_branch_diameter_organic",
            crate::OptionValueKind::Float,
            "2",
            &["PrintConfig.hpp:1036", "PrintConfig.cpp:6380-6388"][..],
        ),
        (
            "tree_support_wall_count",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:1014", "PrintConfig.cpp:6390-6397"][..],
        ),
        (
            "tree_support_with_infill",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.cpp:6399-6404"][..],
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
