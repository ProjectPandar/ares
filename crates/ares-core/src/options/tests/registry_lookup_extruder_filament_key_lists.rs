#[test]
fn exposes_extruder_and_filament_key_lists_for_ui_consumers() {
    assert_eq!(crate::extruder_option_keys()[0], "extruder_type");
    assert_eq!(
        crate::extruder_option_keys().last(),
        Some(&"long_retractions_when_cut")
    );
    assert_eq!(crate::extruder_retract_keys()[0], "deretraction_speed");
    assert_eq!(crate::extruder_retract_keys().last(), Some(&"z_hop_types"));
    assert_eq!(crate::filament_option_keys()[0], "filament_diameter");
    assert_eq!(
        crate::filament_option_keys().last(),
        Some(&"long_retractions_when_cut")
    );
    assert_eq!(crate::filament_retract_keys()[0], "deretraction_speed");
    assert_eq!(crate::filament_retract_keys().last(), Some(&"z_hop_types"));
}
