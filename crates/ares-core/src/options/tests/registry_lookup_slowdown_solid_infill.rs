#[test]
fn exposes_slowdown_solid_infill_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "minimum_sparse_infill_area",
            crate::OptionValueKind::Float,
            "15",
        ),
        (
            "slow_down_layer_time",
            crate::OptionValueKind::Floats,
            "5",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
