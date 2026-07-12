use crate::{OptionValueKind, option_definition};

#[test]
fn ironing_and_zaa_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "ironing_angle",
            OptionValueKind::Float,
            "0",
        ),
        (
            "ironing_angle_fixed",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "ironing_expansion",
            OptionValueKind::Float,
            "0",
        ),
        (
            "ironing_flow",
            OptionValueKind::Percent,
            "10",
        ),
        (
            "ironing_inset",
            OptionValueKind::Float,
            "0",
        ),
        (
            "ironing_pattern",
            OptionValueKind::Enum,
            "rectilinear",
        ),
        (
            "ironing_spacing",
            OptionValueKind::Float,
            "0.1",
        ),
        (
            "ironing_speed",
            OptionValueKind::Float,
            "20",
        ),
        (
            "ironing_type",
            OptionValueKind::Enum,
            "no ironing",
        ),
        (
            "zaa_dont_alternate_fill_direction",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "zaa_enabled",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "zaa_min_z",
            OptionValueKind::Float,
            "0.05",
        ),
        (
            "zaa_minimize_perimeter_height",
            OptionValueKind::Float,
            "35",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
