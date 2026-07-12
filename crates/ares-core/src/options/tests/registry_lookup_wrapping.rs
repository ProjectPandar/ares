use crate::{OptionValueKind, option_definition};

#[test]
fn wrapping_detection_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_wrapping_detection",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1348", "PrintConfig.cpp:3987-3991"][..],
        ),
        (
            "wrapping_detection_layers",
            OptionValueKind::Int,
            "20",
            &["PrintConfig.hpp:1349", "PrintConfig.cpp:3993-3998"][..],
        ),
        (
            "wrapping_exclude_area",
            OptionValueKind::Points,
            "0x0",
            &["PrintConfig.hpp:1350", "PrintConfig.cpp:4000-4005"][..],
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
