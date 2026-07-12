#[test]
fn exposes_raft_resolution_retraction_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "raft_contact_distance",
            crate::OptionValueKind::Float,
            "0.1",
        ),
        (
            "raft_expansion",
            crate::OptionValueKind::Float,
            "1.5",
        ),
        (
            "raft_first_layer_density",
            crate::OptionValueKind::Percent,
            "90",
        ),
        (
            "raft_first_layer_expansion",
            crate::OptionValueKind::Float,
            "2.0",
        ),
        (
            "raft_layers",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "resolution",
            crate::OptionValueKind::Float,
            "0.01",
        ),
        (
            "retract_before_wipe",
            crate::OptionValueKind::Percents,
            "100",
        ),
        (
            "retract_when_changing_layer",
            crate::OptionValueKind::Bools,
            "false",
        ),
        (
            "retraction_minimum_travel",
            crate::OptionValueKind::Floats,
            "2",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
