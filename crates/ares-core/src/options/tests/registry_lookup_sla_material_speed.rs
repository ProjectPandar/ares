#[test]
fn exposes_sla_material_speed_option_definition_lookup() {
    let definition = crate::option_definition("material_print_speed").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Enum);
    assert_eq!(definition.default_value, "fast");
}
