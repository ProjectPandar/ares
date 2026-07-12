#[test]
fn exposes_process_gcode_utility_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enable_arc_fitting",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "enable_power_loss_recovery",
            crate::OptionValueKind::Enum,
            "printer_configuration",
        ),
        (
            "filter_out_gap_fill",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "gap_infill_speed",
            crate::OptionValueKind::Float,
            "30",
        ),
        (
            "gcode_add_line_number",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "precise_z_height",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "scan_first_layer",
            crate::OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
