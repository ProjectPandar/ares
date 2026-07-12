#[test]
fn exposes_relative_e_and_wall_generator_option_definition_lookup() {
    let relative_e = crate::option_definition("use_relative_e_distances").unwrap();
    assert_eq!(relative_e.kind, crate::OptionValueKind::Bool);
    assert_eq!(relative_e.default_value, "true");

    let wall_generator = crate::option_definition("wall_generator").unwrap();
    assert_eq!(wall_generator.kind, crate::OptionValueKind::Enum);
    assert_eq!(wall_generator.default_value, "arachne");
}
