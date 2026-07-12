#[test]
fn exposes_strength_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "align_infill_direction_to_model",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1106", "PrintConfig.cpp:2891-2896"][..],
        ),
        (
            "extra_solid_infills",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1107", "PrintConfig.cpp:2898-2903"][..],
        ),
        (
            "fill_multiline",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1135", "PrintConfig.cpp:2906-2913"][..],
        ),
        (
            "gyroid_optimized",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1136", "PrintConfig.cpp:2915-2926"][..],
        ),
        (
            "infill_direction",
            crate::OptionValueKind::Float,
            "45",
            &["PrintConfig.hpp:1095", "PrintConfig.cpp:2861-2869"][..],
        ),
        (
            "infill_anchor",
            crate::OptionValueKind::FloatOrPercent,
            "400%",
            &["PrintConfig.hpp:1195", "PrintConfig.cpp:3017-3043"][..],
        ),
        (
            "infill_anchor_max",
            crate::OptionValueKind::FloatOrPercent,
            "20",
            &["PrintConfig.hpp:1196", "PrintConfig.cpp:3045-3066"][..],
        ),
        (
            "infill_overhang_angle",
            crate::OptionValueKind::Float,
            "60",
            &["PrintConfig.hpp:1105", "PrintConfig.cpp:3007-3015"][..],
        ),
        (
            "lateral_lattice_angle_1",
            crate::OptionValueKind::Float,
            "-45",
            &["PrintConfig.hpp:1103", "PrintConfig.cpp:2987-2995"][..],
        ),
        (
            "lateral_lattice_angle_2",
            crate::OptionValueKind::Float,
            "45",
            &["PrintConfig.hpp:1104", "PrintConfig.cpp:2997-3005"][..],
        ),
        (
            "solid_infill_direction",
            crate::OptionValueKind::Float,
            "45",
            &["PrintConfig.hpp:1096", "PrintConfig.cpp:2871-2879"][..],
        ),
        (
            "sparse_infill_density",
            crate::OptionValueKind::Percent,
            "20",
            &["PrintConfig.hpp:1101", "PrintConfig.cpp:2881-2889"][..],
        ),
        (
            "sparse_infill_pattern",
            crate::OptionValueKind::Enum,
            "crosshatch",
            &["PrintConfig.hpp:1102", "PrintConfig.cpp:2928-2985"][..],
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
