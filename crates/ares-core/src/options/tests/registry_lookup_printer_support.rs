use crate::{OptionValueKind, option_definition};

#[test]
fn fan_pwm_cost_and_printer_support_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "part_cooling_fan_min_pwm",
            OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:1316", "PrintConfig.cpp:3740-3760"][..],
        ),
        (
            "support_air_filtration",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1405", "PrintConfig.cpp:3779-3783"][..],
        ),
        (
            "support_chamber_temp_control",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1407", "PrintConfig.cpp:3771-3777"][..],
        ),
        (
            "time_cost",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1357", "PrintConfig.cpp:3763-3769"][..],
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
