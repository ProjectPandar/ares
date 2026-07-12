#[test]
fn exposes_sla_pad_zero_elevation_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "pad_around_object",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "pad_around_object_everywhere",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "pad_object_connector_penetration",
            crate::OptionValueKind::Float,
            "0.3",
        ),
        (
            "pad_object_connector_stride",
            crate::OptionValueKind::Float,
            "10",
        ),
        (
            "pad_object_connector_width",
            crate::OptionValueKind::Float,
            "0.5",
        ),
        (
            "pad_object_gap",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
