use crate::{OptionValueKind, option_definition};

#[test]
fn fan_pwm_cost_and_printer_support_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "part_cooling_fan_min_pwm",
            OptionValueKind::Int,
            "0",
        ),
        (
            "support_air_filtration",
            OptionValueKind::Bool,
            "true",
        ),
        (
            "support_chamber_temp_control",
            OptionValueKind::Bool,
            "true",
        ),
        (
            "time_cost",
            OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
