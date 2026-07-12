#[test]
fn exposes_retraction_cut_toolchange_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "enable_long_retraction_when_cut",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "long_retractions_when_cut",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "long_retractions_when_ec",
            crate::OptionValueKind::BoolsNullable,
            "false",
        ),
        (
            "retract_length_toolchange",
            crate::OptionValueKind::Floats,
            "10",
        ),
        (
            "retraction_distances_when_cut",
            crate::OptionValueKind::Floats,
            "18",
        ),
        (
            "retraction_distances_when_ec",
            crate::OptionValueKind::FloatsNullable,
            "10",
        ),
        (
            "retraction_length",
            crate::OptionValueKind::Floats,
            "0.8",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
