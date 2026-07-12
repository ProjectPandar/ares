use super::super::super::{OptionValueKind, option_definition};

#[test]
fn brim_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "brim_ears",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.cpp:1665"][..],
        ),
        (
            "brim_ears_detection_length",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:925", "PrintConfig.cpp:1684"][..],
        ),
        (
            "brim_ears_max_angle",
            OptionValueKind::Float,
            "125",
            &["PrintConfig.hpp:926", "PrintConfig.cpp:1672"][..],
        ),
        (
            "brim_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:921", "PrintConfig.cpp:1637"][..],
        ),
        (
            "brim_use_efc_outline",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:922", "PrintConfig.cpp:1648"][..],
        ),
        (
            "combine_brims",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1619", "PrintConfig.cpp:1658"][..],
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
