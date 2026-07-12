#[test]
fn exposes_sla_material_speed_option_definition_lookup() {
    let definition = crate::option_definition("material_print_speed").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Enum);
    assert_eq!(definition.default_value, "fast");
    for fragment in [
        "PrintConfig.hpp:1805",
        "PrintConfig.hpp:1821",
        "PrintConfig.cpp:413-417",
        "PrintConfig.cpp:7855-7864",
    ] {
        assert!(definition.source.contains(fragment));
    }
}
