use super::super::super::{OptionValueKind, option_definition};

#[test]
fn compatible_profile_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("activate_air_filtration", OptionValueKind::Bools, "false"),
        (
            "activate_air_filtration_during_print",
            OptionValueKind::Bools,
            "true",
        ),
        (
            "activate_air_filtration_on_completion",
            OptionValueKind::Bools,
            "true",
        ),
        ("close_fan_the_first_x_layers", OptionValueKind::Ints, "1"),
        (
            "complete_print_exhaust_fan_speed",
            OptionValueKind::Ints,
            "80",
        ),
        (
            "compatible_machine_expression_group",
            OptionValueKind::Strings,
            "",
        ),
        ("compatible_printers", OptionValueKind::Strings, ""),
        ("compatible_printers_condition", OptionValueKind::String, ""),
        ("compatible_prints", OptionValueKind::Strings, ""),
        ("compatible_prints_condition", OptionValueKind::String, ""),
        (
            "compatible_process_expression_group",
            OptionValueKind::Strings,
            "",
        ),
        ("default_acceleration", OptionValueKind::Float, "500"),
        ("default_filament_profile", OptionValueKind::Strings, ""),
        ("default_print_profile", OptionValueKind::String, ""),
        ("different_settings_to_system", OptionValueKind::Strings, ""),
        (
            "machine_end_gcode",
            OptionValueKind::String,
            "M104 S0 ; turn off temperature\nG28 X0  ; home X axis\nM84     ; disable motors\n",
        ),
        ("printing_by_object_gcode", OptionValueKind::String, ""),
        ("filament_end_gcode", OptionValueKind::Strings, " "),
        (
            "during_print_exhaust_fan_speed",
            OptionValueKind::Ints,
            "60",
        ),
        ("print_compatible_printers", OptionValueKind::Strings, ""),
        ("print_order", OptionValueKind::Enum, "default"),
        ("print_sequence", OptionValueKind::Enum, "by layer"),
        (
            "slow_down_for_layer_cooling",
            OptionValueKind::Bools,
            "true",
        ),
        ("upward_compatible_machine", OptionValueKind::Strings, ""),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
