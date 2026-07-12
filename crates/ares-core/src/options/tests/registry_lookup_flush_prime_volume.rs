#[test]
fn exposes_flush_prime_volume_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "flush_volumes_vector",
            crate::OptionValueKind::Floats,
            "140,140,140,140,140,140,140,140",
            &["PrintConfig.hpp:1591", "PrintConfig.cpp:6659-6667"][..],
        ),
        (
            "flush_volumes_matrix",
            crate::OptionValueKind::Floats,
            "0,280,280,280,280,0,280,280,280,280,0,280,280,280,280,0",
            &["PrintConfig.hpp:1590", "PrintConfig.cpp:6669-6677"][..],
        ),
        (
            "flush_multiplier",
            crate::OptionValueKind::Floats,
            "0.3",
            &["PrintConfig.hpp:1608", "PrintConfig.cpp:6679-6683"][..],
        ),
        (
            "prime_volume",
            crate::OptionValueKind::Float,
            "45",
            &["PrintConfig.hpp:1607", "PrintConfig.cpp:6686-6692"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
