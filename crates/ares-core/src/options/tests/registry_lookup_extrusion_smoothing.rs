#[test]
fn exposes_fan_max_extrusion_smoothing_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1364", "PrintConfig.cpp:4643-4648"][..],
        ),
        (
            "fan_max_speed",
            crate::OptionValueKind::Floats,
            "100",
            &["PrintConfig.hpp:1535", "PrintConfig.cpp:4591-4599"][..],
        ),
        (
            "max_layer_height",
            crate::OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1536", "PrintConfig.cpp:4601-4608"][..],
        ),
        (
            "max_volumetric_extrusion_rate_slope",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1362", "PrintConfig.cpp:4610-4629"][..],
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            crate::OptionValueKind::Float,
            "3.0",
            &["PrintConfig.hpp:1363", "PrintConfig.cpp:4631-4641"][..],
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
