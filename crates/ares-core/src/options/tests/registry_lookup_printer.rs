use crate::{OptionValueKind, option_definition};

#[test]
fn printer_structure_and_fan_speedup_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "auxiliary_fan",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1404", "PrintConfig.cpp:3704-3708"][..],
        ),
        (
            "best_object_pos",
            OptionValueKind::Point,
            "0.5x0.5",
            &["PrintConfig.hpp:1541", "PrintConfig.cpp:3698-3702"][..],
        ),
        (
            "fan_kickstart",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1310", "PrintConfig.cpp:3729-3738"][..],
        ),
        (
            "fan_speedup_overhangs",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1311", "PrintConfig.cpp:3723-3727"][..],
        ),
        (
            "fan_speedup_time",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1312", "PrintConfig.cpp:3710-3721"][..],
        ),
        (
            "printer_structure",
            OptionValueKind::Enum,
            "undefine",
            &[
                "PrintConfig.hpp:357-363",
                "PrintConfig.hpp:1406",
                "PrintConfig.cpp:494-501",
                "PrintConfig.cpp:3681-3696",
            ][..],
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
