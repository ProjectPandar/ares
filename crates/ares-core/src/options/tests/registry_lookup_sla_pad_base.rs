#[test]
fn exposes_sla_pad_base_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "pad_brim_size",
            crate::OptionValueKind::Float,
            "1.6",
        ),
        (
            "pad_enable",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "pad_max_merge_distance",
            crate::OptionValueKind::Float,
            "50",
        ),
        (
            "pad_wall_height",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "pad_wall_slope",
            crate::OptionValueKind::Float,
            "90",
        ),
        (
            "pad_wall_thickness",
            crate::OptionValueKind::Float,
            "2",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
