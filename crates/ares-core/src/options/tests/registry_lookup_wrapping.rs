use crate::{OptionValueKind, option_definition};

#[test]
fn wrapping_detection_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "enable_wrapping_detection",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "wrapping_detection_layers",
            OptionValueKind::Int,
            "20",
        ),
        (
            "wrapping_exclude_area",
            OptionValueKind::Points,
            "0x0",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
