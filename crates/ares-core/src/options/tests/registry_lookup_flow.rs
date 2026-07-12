#[test]
fn exposes_flow_ratio_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "filament_flow_ratio",
            crate::OptionValueKind::FloatsNullable,
            "1",
        ),
        ("print_flow_ratio", crate::OptionValueKind::Float, "1"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
