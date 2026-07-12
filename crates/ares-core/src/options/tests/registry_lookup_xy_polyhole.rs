fn assert_definition(
    key: &str,
    kind: crate::OptionValueKind,
    default_value: &str,
    hpp_line: &str,
    cpp_lines: &str,
) {
    let definition = crate::option_definition(key).unwrap();

    assert_eq!(definition.kind, kind);
    assert_eq!(definition.default_value, default_value);
    assert!(definition.source.contains(hpp_line));
    assert!(definition.source.contains(cpp_lines));
}

#[test]
fn exposes_xy_compensation_and_polyhole_option_definition_lookup() {
    assert_definition(
        "xy_hole_compensation",
        crate::OptionValueKind::Float,
        "0",
        "PrintConfig.hpp:1001",
        "PrintConfig.cpp:6907-6915",
    );
    assert_definition(
        "xy_contour_compensation",
        crate::OptionValueKind::Float,
        "0",
        "PrintConfig.hpp:1002",
        "PrintConfig.cpp:6917-6925",
    );
    assert_definition(
        "hole_to_polyhole",
        crate::OptionValueKind::Bool,
        "false",
        "PrintConfig.hpp:1202",
        "PrintConfig.cpp:6927-6934",
    );
    assert_definition(
        "hole_to_polyhole_threshold",
        crate::OptionValueKind::FloatOrPercent,
        "0.01",
        "PrintConfig.hpp:1203",
        "PrintConfig.cpp:6936-6947",
    );
    assert_definition(
        "hole_to_polyhole_twisted",
        crate::OptionValueKind::Bool,
        "true",
        "PrintConfig.hpp:1204",
        "PrintConfig.cpp:6949-6954",
    );
}
