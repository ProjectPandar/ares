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
fn exposes_wipe_tower_extra_and_idle_temperature_option_definition_lookup() {
    assert_definition(
        "wipe_tower_bridging",
        crate::OptionValueKind::Float,
        "10",
    );
    assert_definition(
        "wipe_tower_extra_spacing",
        crate::OptionValueKind::Percent,
        "100",
    );
    assert_definition(
        "wipe_tower_extra_flow",
        crate::OptionValueKind::Percent,
        "100",
    );
    assert_definition(
        "idle_temperature",
        crate::OptionValueKind::Ints,
        "0",
    );
}
