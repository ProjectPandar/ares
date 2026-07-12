#[test]
fn exposes_wipe_tower_max_purge_speed_option_definition_lookup() {
    let definition = crate::option_definition("wipe_tower_max_purge_speed").unwrap();

    assert_eq!(definition.kind, crate::OptionValueKind::Float);
    assert_eq!(definition.default_value, "90");
    assert!(definition.source.contains("PrintConfig.hpp:1596"));
    assert!(definition.source.contains("PrintConfig.cpp:6746-6757"));
}
