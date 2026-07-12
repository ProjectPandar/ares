#[test]
fn exposes_internal_solid_spiral_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "internal_solid_infill_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1162", "PrintConfig.cpp:5657-5667"][..],
        ),
        (
            "internal_solid_infill_speed",
            crate::OptionValueKind::Float,
            "100",
            &["PrintConfig.hpp:1163", "PrintConfig.cpp:5669-5676"][..],
        ),
        (
            "spiral_finishing_flow_ratio",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1563", "PrintConfig.cpp:5717-5726"][..],
        ),
        (
            "spiral_mode",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1560", "PrintConfig.cpp:5678-5684"][..],
        ),
        (
            "spiral_mode_max_xy_smoothing",
            crate::OptionValueKind::FloatOrPercent,
            "200%",
            &["PrintConfig.hpp:1562", "PrintConfig.cpp:5693-5704"][..],
        ),
        (
            "spiral_mode_smooth",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1561", "PrintConfig.cpp:5686-5691"][..],
        ),
        (
            "spiral_starting_flow_ratio",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1564", "PrintConfig.cpp:5706-5715"][..],
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
