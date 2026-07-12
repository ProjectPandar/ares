#[test]
fn exposes_support_filament_option_definition_lookup() {
    let definition = crate::option_definition("support_filament").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Int);
    assert_eq!(definition.default_value, "0");
    assert!(definition.source.contains("PrintConfig.hpp:959"));
    assert!(definition.source.contains("PrintConfig.cpp:6027-6034"));
}
