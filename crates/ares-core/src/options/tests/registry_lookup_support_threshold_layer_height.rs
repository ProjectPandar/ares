#[test]
fn exposes_support_threshold_layer_height_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "independent_support_layer_height",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "support_threshold_angle",
            crate::OptionValueKind::Int,
            "30",
        ),
        (
            "support_threshold_overlap",
            crate::OptionValueKind::FloatOrPercent,
            "50%",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
