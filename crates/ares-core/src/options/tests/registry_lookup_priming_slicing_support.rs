#[test]
fn exposes_priming_slicing_support_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enable_support",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "single_extruder_multi_material_priming",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "slice_closing_radius",
            crate::OptionValueKind::Float,
            "0.049",
        ),
        (
            "slicing_mode",
            crate::OptionValueKind::Enum,
            "regular",
        ),
        (
            "z_offset",
            crate::OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
