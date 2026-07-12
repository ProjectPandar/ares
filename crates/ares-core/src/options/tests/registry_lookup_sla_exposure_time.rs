#[test]
fn exposes_sla_exposure_time_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "exposure_time",
            crate::OptionValueKind::Float,
            "10",
        ),
        (
            "faded_layers",
            crate::OptionValueKind::Int,
            "10",
        ),
        (
            "initial_exposure_time",
            crate::OptionValueKind::Float,
            "15",
        ),
        (
            "max_exposure_time",
            crate::OptionValueKind::Float,
            "100",
        ),
        (
            "max_initial_exposure_time",
            crate::OptionValueKind::Float,
            "150",
        ),
        (
            "min_exposure_time",
            crate::OptionValueKind::Float,
            "0",
        ),
        (
            "min_initial_exposure_time",
            crate::OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
