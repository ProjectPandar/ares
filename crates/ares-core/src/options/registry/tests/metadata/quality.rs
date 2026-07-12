use super::super::super::{OptionValueKind, option_definition};

#[test]
fn one_wall_quality_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "precise_outer_wall",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1188", "PrintConfig.cpp:1404"][..],
        ),
        (
            "only_one_wall_top",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1176", "PrintConfig.cpp:1411"][..],
        ),
        (
            "min_width_top_surface",
            OptionValueKind::FloatOrPercent,
            "300%",
            &["PrintConfig.hpp:1179", "PrintConfig.cpp:1418"][..],
        ),
        (
            "only_one_wall_first_layer",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1180", "PrintConfig.cpp:1433"][..],
        ),
        (
            "overhang_reverse",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1205", "PrintConfig.cpp:1446"][..],
        ),
        (
            "overhang_reverse_internal_only",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1206", "PrintConfig.cpp:1454"][..],
        ),
        (
            "counterbore_hole_bridging",
            OptionValueKind::Enum,
            "none",
            &[
                "PrintConfig.hpp:401",
                "PrintConfig.hpp:1208",
                "PrintConfig.cpp:551",
                "PrintConfig.cpp:1467",
            ][..],
        ),
        (
            "overhang_reverse_threshold",
            OptionValueKind::FloatOrPercent,
            "50%",
            &["PrintConfig.hpp:1207", "PrintConfig.cpp:1485"][..],
        ),
        (
            "extra_perimeters_on_overhangs",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1200", "PrintConfig.cpp:1439"][..],
        ),
        (
            "bridge_no_support",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:928", "PrintConfig.cpp:1847-1853"][..],
        ),
        (
            "dont_filter_internal_bridges",
            OptionValueKind::Enum,
            "disabled",
            &[
                "PrintConfig.hpp:231",
                "PrintConfig.hpp:988",
                "PrintConfig.cpp:377-382",
                "PrintConfig.cpp:1902-1928",
            ][..],
        ),
        (
            "enable_extra_bridge_layer",
            OptionValueKind::Enum,
            "disabled",
            &[
                "PrintConfig.hpp:236",
                "PrintConfig.hpp:990",
                "PrintConfig.cpp:384-390",
                "PrintConfig.cpp:1871-1900",
            ][..],
        ),
        (
            "max_bridge_length",
            OptionValueKind::Float,
            "10",
            &["PrintConfig.hpp:932", "PrintConfig.cpp:1931-1938"][..],
        ),
        (
            "ensure_vertical_shell_thickness",
            OptionValueKind::Enum,
            "ensure_all",
            &[
                "PrintConfig.hpp:223",
                "PrintConfig.hpp:1087",
                "PrintConfig.cpp:368-374",
                "PrintConfig.cpp:1967-1984",
            ][..],
        ),
        (
            "top_surface_pattern",
            OptionValueKind::Enum,
            "monotonicline",
            &[
                "PrintConfig.hpp:87",
                "PrintConfig.hpp:1090",
                "PrintConfig.cpp:225-255",
                "PrintConfig.cpp:1986-2007",
            ][..],
        ),
        (
            "bottom_surface_pattern",
            OptionValueKind::Enum,
            "monotonic",
            &[
                "PrintConfig.hpp:87",
                "PrintConfig.hpp:1091",
                "PrintConfig.cpp:225-255",
                "PrintConfig.cpp:2009-2016",
            ][..],
        ),
        (
            "internal_solid_infill_pattern",
            OptionValueKind::Enum,
            "monotonic",
            &[
                "PrintConfig.hpp:87",
                "PrintConfig.hpp:1092",
                "PrintConfig.cpp:225-255",
                "PrintConfig.cpp:2018-2025",
            ][..],
        ),
        (
            "small_perimeter_threshold",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1192", "PrintConfig.cpp:2061-2068"][..],
        ),
        (
            "wall_sequence",
            OptionValueKind::Enum,
            "inner wall/outer wall",
            &[
                "PrintConfig.hpp:132",
                "PrintConfig.hpp:1209",
                "PrintConfig.cpp:277-283",
                "PrintConfig.cpp:2070-2091",
            ][..],
        ),
        (
            "wall_direction",
            OptionValueKind::Enum,
            "ccw",
            &[
                "PrintConfig.hpp:140",
                "PrintConfig.hpp:1212",
                "PrintConfig.cpp:286-290",
                "PrintConfig.cpp:2100-2110",
            ][..],
        ),
        (
            "thick_bridges",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:986", "PrintConfig.cpp:1855-1861"][..],
        ),
        (
            "thick_internal_bridges",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:987", "PrintConfig.cpp:1863-1869"][..],
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
