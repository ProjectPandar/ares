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
fn exposes_wipe_tower_rib_and_filament_option_definition_lookup() {
    assert_definition(
        "wipe_tower_extra_rib_length",
        crate::OptionValueKind::Float,
        "0",
        "PrintConfig.hpp:1598",
        "PrintConfig.cpp:6775-6782",
    );
    assert_definition(
        "wipe_tower_filament",
        crate::OptionValueKind::Int,
        "0",
        "PrintConfig.hpp:1601",
        "PrintConfig.cpp:6800-6808",
    );
    assert_definition(
        "wipe_tower_fillet_wall",
        crate::OptionValueKind::Bool,
        "true",
        "PrintConfig.hpp:1600",
        "PrintConfig.cpp:6793-6797",
    );
    assert_definition(
        "wipe_tower_rib_width",
        crate::OptionValueKind::Float,
        "8",
        "PrintConfig.hpp:1599",
        "PrintConfig.cpp:6784-6791",
    );
}
