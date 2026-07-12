#[test]
fn exposes_skirt_draft_shield_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "draft_shield",
            crate::OptionValueKind::Enum,
            "disabled",
        ),
        (
            "min_skirt_length",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "single_loop_draft_shield",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "skirt_distance",
            crate::OptionValueKind::Float,
            "2",
        ),
        (
            "skirt_height",
            crate::OptionValueKind::Int,
            "1",
        ),
        (
            "skirt_loops",
            crate::OptionValueKind::Int,
            "1",
        ),
        (
            "skirt_speed",
            crate::OptionValueKind::Float,
            "50",
        ),
        (
            "skirt_start_angle",
            crate::OptionValueKind::Float,
            "-135",
        ),
        (
            "skirt_type",
            crate::OptionValueKind::Enum,
            "combined",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
