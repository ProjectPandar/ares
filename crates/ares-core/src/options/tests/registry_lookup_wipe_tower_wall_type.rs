#[test]
fn exposes_wipe_tower_wall_type_option_definition_lookup() {
    let definition = crate::option_definition("wipe_tower_wall_type").unwrap();

    assert_eq!(definition.kind, crate::OptionValueKind::Enum);
    assert_eq!(definition.default_value, "rib");
}
