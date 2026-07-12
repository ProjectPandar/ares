use super::super::super::{OptionValueKind, option_definition};

#[test]
fn pressure_advance_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("adaptive_pressure_advance", OptionValueKind::Bools, "false"),
        (
            "adaptive_pressure_advance_bridges",
            OptionValueKind::Floats,
            "0",
        ),
        (
            "adaptive_pressure_advance_model",
            OptionValueKind::Strings,
            "0,0,0\n0,0,0",
        ),
        (
            "adaptive_pressure_advance_overhangs",
            OptionValueKind::Bools,
            "false",
        ),
        ("enable_pressure_advance", OptionValueKind::Bools, "false"),
        ("pressure_advance", OptionValueKind::Floats, "0.02"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
