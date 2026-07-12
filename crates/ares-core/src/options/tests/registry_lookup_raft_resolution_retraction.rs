#[test]
fn exposes_raft_resolution_retraction_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "raft_contact_distance",
            crate::OptionValueKind::Float,
            "0.1",
            &["PrintConfig.hpp:939", "PrintConfig.cpp:4988-4997"][..],
        ),
        (
            "raft_expansion",
            crate::OptionValueKind::Float,
            "1.5",
            &["PrintConfig.hpp:940", "PrintConfig.cpp:4999-5006"][..],
        ),
        (
            "raft_first_layer_density",
            crate::OptionValueKind::Percent,
            "90",
            &["PrintConfig.hpp:941", "PrintConfig.cpp:5008-5016"][..],
        ),
        (
            "raft_first_layer_expansion",
            crate::OptionValueKind::Float,
            "2.0",
            &["PrintConfig.hpp:942", "PrintConfig.cpp:5018-5026"][..],
        ),
        (
            "raft_layers",
            crate::OptionValueKind::Int,
            "0",
            &[
                "PrintConfigConstants.hpp:6",
                "PrintConfig.hpp:943",
                "PrintConfig.cpp:5028-5037",
            ][..],
        ),
        (
            "resolution",
            crate::OptionValueKind::Float,
            "0.01",
            &["PrintConfig.hpp:1549", "PrintConfig.cpp:5039-5046"][..],
        ),
        (
            "retract_before_wipe",
            crate::OptionValueKind::Percents,
            "100",
            &["PrintConfig.hpp:1367", "PrintConfig.cpp:5055-5060"][..],
        ),
        (
            "retract_when_changing_layer",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1551", "PrintConfig.cpp:5062-5066"][..],
        ),
        (
            "retraction_minimum_travel",
            crate::OptionValueKind::Floats,
            "2",
            &["PrintConfig.hpp:1550", "PrintConfig.cpp:5048-5053"][..],
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
