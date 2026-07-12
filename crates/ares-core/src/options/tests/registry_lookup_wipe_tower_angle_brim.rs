#[test]
fn exposes_wipe_tower_angle_brim_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "wipe_tower_rotation_angle",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "prime_tower_brim_width",
            crate::OptionValueKind::Float,
            "3",
        ),
        (
            "wipe_tower_cone_angle",
            crate::OptionValueKind::Float,
            "30",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
