#[test]
fn exposes_wipe_speed_loop_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "role_based_wipe_speed",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "wipe_before_external_loop",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "wipe_on_loops",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "wipe_speed",
            crate::OptionValueKind::FloatOrPercent,
            "80%",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
