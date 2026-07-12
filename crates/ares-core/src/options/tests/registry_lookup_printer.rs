use crate::{OptionValueKind, option_definition};

#[test]
fn printer_structure_and_fan_speedup_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "auxiliary_fan",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "best_object_pos",
            OptionValueKind::Point,
            "0.5x0.5",
        ),
        (
            "fan_kickstart",
            OptionValueKind::Float,
            "0",
        ),
        (
            "fan_speedup_overhangs",
            OptionValueKind::Bool,
            "true",
        ),
        (
            "fan_speedup_time",
            OptionValueKind::Float,
            "0",
        ),
        (
            "printer_structure",
            OptionValueKind::Enum,
            "undefine",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
