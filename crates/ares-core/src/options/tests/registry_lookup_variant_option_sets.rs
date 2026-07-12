#[test]
fn exposes_print_config_variant_option_key_sets_for_ui_consumers() {
    assert_eq!(crate::print_options_with_variant().len(), 2);
    assert_eq!(
        crate::print_options_with_variant().last(),
        Some(&"print_extruder_variant")
    );
    assert_eq!(
        crate::filament_options_with_variant().first(),
        Some(&"activate_air_filtration")
    );
    assert_eq!(
        crate::filament_options_with_variant().last(),
        Some(&"volumetric_speed_coefficients")
    );
    assert_eq!(
        crate::printer_extruder_options().first(),
        Some(&"default_nozzle_volume_type")
    );
    assert_eq!(
        crate::printer_options_with_variant_1().first(),
        Some(&"deretraction_speed")
    );
    assert_eq!(
        crate::printer_options_with_variant_2().last(),
        Some(&"machine_max_speed_z")
    );
}
