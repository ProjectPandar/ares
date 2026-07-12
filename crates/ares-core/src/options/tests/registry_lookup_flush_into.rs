#[test]
fn exposes_flush_into_option_definition_lookup() {
    for (key, default_value) in [
        (
            "flush_into_infill",
            "false",
        ),
        (
            "flush_into_objects",
            "false",
        ),
        (
            "flush_into_support",
            "true",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Bool);
        assert_eq!(definition.default_value, default_value);
    }
}
