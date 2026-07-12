#[test]
fn exposes_restart_speed_seam_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "bbl_calib_mark_logo",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "deretraction_speed",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "disable_m73",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "retract_restart_extra",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "retract_restart_extra_toolchange",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "retraction_speed",
            crate::OptionValueKind::Floats,
            "30",
        ),
        (
            "seam_gap",
            crate::OptionValueKind::FloatOrPercent,
            "10%",
        ),
        (
            "seam_position",
            crate::OptionValueKind::Enum,
            "aligned",
        ),
        (
            "staggered_inner_seams",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "use_firmware_retraction",
            crate::OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
