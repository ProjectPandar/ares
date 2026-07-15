#[test]
fn exposes_overhang_wall_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "detect_overhang_wall",
            crate::OptionValueKind::Bool,
            "true",
        ),
        (
            "inner_wall_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
        ),
        (
            "inner_wall_speed",
            crate::OptionValueKind::Float,
            "60",
        ),
        (
            "make_overhang_printable",
            crate::OptionValueKind::Bool,
            "false",
        ),
        (
            "make_overhang_printable_angle",
            crate::OptionValueKind::Float,
            "55",
        ),
        (
            "make_overhang_printable_hole_size",
            crate::OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
