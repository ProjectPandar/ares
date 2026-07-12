#[test]
fn exposes_extruder_variant_id_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "extruder_ams_count",
            crate::OptionValueKind::Strings,
            "",
        ),
        (
            "extruder_variant_list",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
        ),
        (
            "filament_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
        ),
        (
            "filament_self_index",
            crate::OptionValueKind::Ints,
            "1",
        ),
        (
            "master_extruder_id",
            crate::OptionValueKind::Int,
            "1",
        ),
        (
            "print_extruder_id",
            crate::OptionValueKind::Ints,
            "1",
        ),
        (
            "print_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
        ),
        (
            "printer_extruder_id",
            crate::OptionValueKind::Ints,
            "1",
        ),
        (
            "printer_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
