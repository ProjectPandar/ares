#[test]
fn exposes_sla_axis_absolute_correction_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "absolute_correction",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "relative_correction_x",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "relative_correction_y",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "relative_correction_z",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
