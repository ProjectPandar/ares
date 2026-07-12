#[test]
fn exposes_fan_max_extrusion_smoothing_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "extrusion_rate_smoothing_external_perimeter_only",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "fan_max_speed",
            crate::OptionValueKind::Floats,
            "100",
        ),
        (
            "max_layer_height",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "max_volumetric_extrusion_rate_slope",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            crate::OptionValueKind::Float,
            "3.0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
