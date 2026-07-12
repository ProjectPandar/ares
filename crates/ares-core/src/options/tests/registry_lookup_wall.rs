#[test]
fn exposes_wall_and_small_perimeter_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "small_perimeter_speed",
            crate::OptionValueKind::FloatOrPercent,
            "50%",
        ),
        (
            "small_perimeter_threshold",
            crate::OptionValueKind::Float,
            "0",
        ),
        ("wall_direction", crate::OptionValueKind::Enum, "ccw"),
        (
            "wall_sequence",
            crate::OptionValueKind::Enum,
            "inner wall/outer wall",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
