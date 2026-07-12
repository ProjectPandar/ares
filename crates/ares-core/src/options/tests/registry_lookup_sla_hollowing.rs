#[test]
fn exposes_sla_hollowing_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "hollowing_closing_distance",
            crate::OptionValueKind::Float,
            "2",
            &["PrintConfig.hpp:1802", "PrintConfig.cpp:7845-7853"][..],
        ),
        (
            "hollowing_enable",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1791", "PrintConfig.cpp:7819-7824"][..],
        ),
        (
            "hollowing_min_thickness",
            crate::OptionValueKind::Float,
            "3",
            &["PrintConfig.hpp:1796", "PrintConfig.cpp:7826-7834"][..],
        ),
        (
            "hollowing_quality",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1799", "PrintConfig.cpp:7836-7843"][..],
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
