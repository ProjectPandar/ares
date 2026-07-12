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
fn exposes_wipe_tower_rib_and_filament_option_definition_lookup() {
    assert_definition(
        "wipe_tower_extra_rib_length",
        crate::OptionValueKind::Float,
        "0",
    );
    assert_definition(
        "wipe_tower_filament",
        crate::OptionValueKind::Int,
        "0",
    );
    assert_definition(
        "wipe_tower_fillet_wall",
        crate::OptionValueKind::Bool,
        "true",
    );
    assert_definition(
        "wipe_tower_rib_width",
        crate::OptionValueKind::Float,
        "8",
    );
}
