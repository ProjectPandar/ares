use crate::{OptionValueKind, option_definition};

#[test]
fn gcode_flavor_and_object_label_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "exclude_object",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "gcode_comments",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "gcode_flavor",
            OptionValueKind::Enum,
            "marlin",
        ),
        (
            "gcode_label_objects",
            OptionValueKind::Bool,
            "true",
        ),
        (
            "pellet_modded_printer",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "support_multi_bed_types",
            OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
