#[test]
fn exposes_host_mmu_parking_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "cooling_tube_length",
            crate::OptionValueKind::Float,
            "5",
        ),
        (
            "cooling_tube_retraction",
            crate::OptionValueKind::Float,
            "91.5",
        ),
        (
            "high_current_on_filament_swap",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "host_type",
            crate::OptionValueKind::Enum,
            "octoprint",
        ),
        (
            "notes",
            crate::OptionValueKind::String,
            "",
        ),
        (
            "nozzle_volume",
            crate::OptionValueKind::FloatsNullable,
            "0.0",
        ),
        (
            "parking_pos_retraction",
            crate::OptionValueKind::Float,
            "92",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
