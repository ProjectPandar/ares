#[test]
fn exposes_support_skip_flush_option_definition_lookup() {
    let definition = crate::option_definition("support_object_skip_flush").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Bool);
    assert_eq!(definition.default_value, "false");
}
