#[test]
fn exposes_sla_relative_correction_option_definition_lookup() {
    let definition = crate::option_definition("relative_correction").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Floats);
    assert_eq!(definition.default_value, "1");
}
