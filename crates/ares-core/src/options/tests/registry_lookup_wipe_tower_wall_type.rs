#[test]
fn exposes_wipe_tower_wall_type_option_definition_lookup() {
    let definition = crate::option_definition("wipe_tower_wall_type").unwrap();

    assert_eq!(definition.kind, crate::OptionValueKind::Enum);
    assert_eq!(definition.default_value, "rib");
    assert!(definition.source.contains("PrintConfig.hpp:405-408"));
    assert!(definition.source.contains("PrintConfig.hpp:1597"));
    assert!(definition.source.contains("PrintConfig.cpp:558-563"));
    assert!(definition.source.contains("PrintConfig.cpp:6759-6773"));
}
