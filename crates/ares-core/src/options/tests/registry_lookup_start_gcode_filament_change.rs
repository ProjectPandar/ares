#[test]
fn exposes_start_gcode_filament_change_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "file_start_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "filament_start_gcode",
            crate::OptionValueKind::Strings,
            " ",
        ),
        (
            "machine_start_gcode",
            crate::OptionValueKind::String,
            "G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n",
        ),
        (
            "manual_filament_change",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "single_extruder_multi_material",
            crate::OptionValueKind::Bool,
            "true",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
