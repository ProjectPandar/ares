#[test]
fn exposes_support_type_placement_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_angle",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:952", "PrintConfig.cpp:5949-5957"][..],
        ),
        (
            "support_critical_regions_only",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:954", "PrintConfig.cpp:5967-5972"][..],
        ),
        (
            "support_object_first_layer_gap",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.cpp:5938-5947"][..],
        ),
        (
            "support_object_xy_distance",
            crate::OptionValueKind::Float,
            "0.35",
            &["PrintConfig.cpp:5927-5936"][..],
        ),
        (
            "support_on_build_plate_only",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:953", "PrintConfig.cpp:5959-5964"][..],
        ),
        (
            "support_remove_small_overhang",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:955", "PrintConfig.cpp:5974-5979"][..],
        ),
        (
            "support_type",
            crate::OptionValueKind::Enum,
            "normal(auto)",
            &[
                "PrintConfig.hpp:195-209",
                "PrintConfig.hpp:950",
                "PrintConfig.cpp:342-348",
                "PrintConfig.cpp:5910-5925",
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
