#[test]
fn exposes_slowdown_solid_infill_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "minimum_sparse_infill_area",
            crate::OptionValueKind::Float,
            "15",
            &["PrintConfig.hpp:1160", "PrintConfig.cpp:5639-5646"][..],
        ),
        (
            "slow_down_layer_time",
            crate::OptionValueKind::Floats,
            "5",
            &["PrintConfig.hpp:1559", "PrintConfig.cpp:5629-5637"][..],
        ),
        (
            "solid_infill_filament",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1161", "PrintConfig.cpp:5648-5655"][..],
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
