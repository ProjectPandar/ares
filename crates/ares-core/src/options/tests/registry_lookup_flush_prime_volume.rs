#[test]
fn exposes_flush_prime_volume_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "flush_volumes_vector",
            crate::OptionValueKind::Floats,
            "140,140,140,140,140,140,140,140",
        ),
        (
            "flush_volumes_matrix",
            crate::OptionValueKind::Floats,
            "0,280,280,280,280,0,280,280,280,280,0,280,280,280,280,0",
        ),
        (
            "flush_multiplier",
            crate::OptionValueKind::Floats,
            "0.3",
        ),
        (
            "prime_volume",
            crate::OptionValueKind::Float,
            "45",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
