use super::super::super::{OptionValueKind, option_definition};

#[test]
fn brim_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("brim_ears", OptionValueKind::Bool, "false"),
        ("brim_ears_detection_length", OptionValueKind::Float, "1"),
        ("brim_ears_max_angle", OptionValueKind::Float, "125"),
        ("brim_flow_ratio", OptionValueKind::Float, "1"),
        ("brim_use_efc_outline", OptionValueKind::Bool, "false"),
        ("combine_brims", OptionValueKind::Bool, "false"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
