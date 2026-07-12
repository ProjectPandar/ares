#[test]
fn exposes_scarf_seam_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "scarf_angle_threshold",
            crate::OptionValueKind::Int,
            "155",
            &["PrintConfig.hpp:1226", "PrintConfig.cpp:5413-5423"][..],
        ),
        (
            "scarf_joint_flow_ratio",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1233", "PrintConfig.cpp:5451-5458"][..],
        ),
        (
            "scarf_joint_speed",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1232", "PrintConfig.cpp:5437-5449"][..],
        ),
        (
            "scarf_overhang_threshold",
            crate::OptionValueKind::Percent,
            "40",
            &["PrintConfig.hpp:1234", "PrintConfig.cpp:5425-5435"][..],
        ),
        (
            "seam_slope_conditional",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1225", "PrintConfig.cpp:5406-5411"][..],
        ),
        (
            "seam_slope_entire_loop",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1228", "PrintConfig.cpp:5471-5476"][..],
        ),
        (
            "seam_slope_inner_walls",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1231", "PrintConfig.cpp:5495-5500"][..],
        ),
        (
            "seam_slope_min_length",
            crate::OptionValueKind::Float,
            "20",
            &["PrintConfig.hpp:1229", "PrintConfig.cpp:5478-5485"][..],
        ),
        (
            "seam_slope_start_height",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1227", "PrintConfig.cpp:5460-5469"][..],
        ),
        (
            "seam_slope_steps",
            crate::OptionValueKind::Int,
            "10",
            &["PrintConfig.hpp:1230", "PrintConfig.cpp:5487-5493"][..],
        ),
        (
            "seam_slope_type",
            crate::OptionValueKind::Enum,
            "none",
            &[
                "PrintConfig.hpp:216-220",
                "PrintConfig.hpp:1224",
                "PrintConfig.cpp:360-365",
                "PrintConfig.cpp:5392-5404",
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
