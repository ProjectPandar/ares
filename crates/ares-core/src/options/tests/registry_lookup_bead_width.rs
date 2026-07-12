#[test]
fn exposes_bead_width_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "initial_layer_min_bead_width",
            crate::OptionValueKind::Percent,
            "85",
        ),
        (
            "min_bead_width",
            crate::OptionValueKind::Percent,
            "85",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
