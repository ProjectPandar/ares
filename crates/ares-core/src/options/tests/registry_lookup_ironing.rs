use crate::{OptionValueKind, option_definition};

#[test]
fn ironing_and_zaa_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "ironing_angle",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1145", "PrintConfig.cpp:4231-4239"][..],
        ),
        (
            "ironing_angle_fixed",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1146", "PrintConfig.cpp:4241-4246"][..],
        ),
        (
            "ironing_expansion",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.cpp:4248-4256"][..],
        ),
        (
            "ironing_flow",
            OptionValueKind::Percent,
            "10",
            &["PrintConfig.hpp:1140", "PrintConfig.cpp:4190-4200"][..],
        ),
        (
            "ironing_inset",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1142", "PrintConfig.cpp:4212-4220"][..],
        ),
        (
            "ironing_pattern",
            OptionValueKind::Enum,
            "rectilinear",
            &[
                "PrintConfig.hpp:87-98",
                "PrintConfig.hpp:1139",
                "PrintConfig.cpp:225-255",
                "PrintConfig.cpp:4178-4188",
            ][..],
        ),
        (
            "ironing_spacing",
            OptionValueKind::Float,
            "0.1",
            &["PrintConfig.hpp:1141", "PrintConfig.cpp:4202-4210"][..],
        ),
        (
            "ironing_speed",
            OptionValueKind::Float,
            "20",
            &["PrintConfig.hpp:1144", "PrintConfig.cpp:4222-4229"][..],
        ),
        (
            "ironing_type",
            OptionValueKind::Enum,
            "no ironing",
            &[
                "PrintConfig.hpp:100-106",
                "PrintConfig.hpp:1138",
                "PrintConfig.cpp:257-263",
                "PrintConfig.cpp:4161-4176",
            ][..],
        ),
        (
            "zaa_dont_alternate_fill_direction",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1238", "PrintConfig.cpp:4277-4282"][..],
        ),
        (
            "zaa_enabled",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1237", "PrintConfig.cpp:4258-4263"][..],
        ),
        (
            "zaa_min_z",
            OptionValueKind::Float,
            "0.05",
            &["PrintConfig.hpp:1239", "PrintConfig.cpp:4284-4293"][..],
        ),
        (
            "zaa_minimize_perimeter_height",
            OptionValueKind::Float,
            "35",
            &["PrintConfig.hpp:1240", "PrintConfig.cpp:4265-4275"][..],
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
