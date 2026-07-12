#[test]
fn exposes_wall_post_process_printer_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "alternate_extra_wall",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "post_process",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "print_settings_id",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "printer_model",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "printer_notes",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "printer_settings_id",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "printer_variant",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "process_change_extrusion_role_gcode",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "wall_loops",
            crate::OptionValueKind::Int,
            "2",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
