#[test]
fn exposes_sla_material_correction_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "material_correction",
            crate::OptionValueKind::Floats,
            "1",
        ),
        (
            "material_correction_x",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "material_correction_y",
            crate::OptionValueKind::Float,
            "1",
        ),
        (
            "material_correction_z",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
