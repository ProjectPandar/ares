#[test]
fn exposes_sla_foot_gamma_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "elefant_foot_min_width",
            crate::OptionValueKind::Float,
            "0.2",
        ),
        (
            "gamma_correction",
            crate::OptionValueKind::Float,
            "1",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
