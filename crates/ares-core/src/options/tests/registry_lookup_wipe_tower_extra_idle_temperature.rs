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
fn exposes_wipe_tower_extra_and_idle_temperature_option_definition_lookup() {
    assert_definition(
        "wipe_tower_bridging",
        crate::OptionValueKind::Float,
        "10",
        "PrintConfig.hpp:1588",
        "PrintConfig.cpp:6872-6877",
    );
    assert_definition(
        "wipe_tower_extra_spacing",
        crate::OptionValueKind::Percent,
        "100",
        "PrintConfig.hpp:1595",
        "PrintConfig.cpp:6879-6886",
    );
    assert_definition(
        "wipe_tower_extra_flow",
        crate::OptionValueKind::Percent,
        "100",
        "PrintConfig.hpp:1589",
        "PrintConfig.cpp:6888-6896",
    );
    assert_definition(
        "idle_temperature",
        crate::OptionValueKind::Ints,
        "0",
        "PrintConfig.hpp:1603",
        "PrintConfig.cpp:6898-6905",
    );
}
