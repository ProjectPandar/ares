#[test]
fn exposes_change_gcode_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "change_extrusion_role_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "change_filament_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "filament_change_extrusion_role_gcode",
            crate::OptionValueKind::Strings,
            "",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
