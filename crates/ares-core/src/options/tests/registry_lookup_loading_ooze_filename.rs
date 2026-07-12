#[test]
fn exposes_loading_ooze_filename_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "extra_loading_move",
            crate::OptionValueKind::Float,
            "-2",
        ),
        (
            "filename_format",
            crate::OptionValueKind::String,
            "{input_filename_base}_{filament_type[initial_tool]}_{print_time}.gcode",
        ),
        (
            "ooze_prevention",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "reduce_infill_retraction",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "start_end_points",
            crate::OptionValueKind::Points,
            "30x-3,54x245",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
