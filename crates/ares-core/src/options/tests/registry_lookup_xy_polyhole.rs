fn assert_definition(
    key: &str,
    kind: crate::OptionValueKind,
    default_value: &str,
) {
    let definition = crate::option_definition(key).unwrap();

    assert_eq!(definition.kind, kind);
    assert_eq!(definition.default_value, default_value);
}

#[test]
fn exposes_xy_compensation_and_polyhole_option_definition_lookup() {
    assert_definition(
        "xy_hole_compensation",
        crate::OptionValueKind::Float,
        "0",
    );
    assert_definition(
        "xy_contour_compensation",
        crate::OptionValueKind::Float,
        "0",
    );
    assert_definition(
        "hole_to_polyhole",
        crate::OptionValueKind::Bool,
        "false",
    );
    assert_definition(
        "hole_to_polyhole_threshold",
        crate::OptionValueKind::FloatOrPercent,
        "0.01",
    );
    assert_definition(
        "hole_to_polyhole_twisted",
        crate::OptionValueKind::Bool,
        "true",
    );
}
