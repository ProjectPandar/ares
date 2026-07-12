#[test]
fn exposes_zhop_extruder_type_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "default_nozzle_volume_type",
            crate::OptionValueKind::Enums,
            "Standard",
        ),
        (
            "extruder_type",
            crate::OptionValueKind::Enums,
            "Direct Drive",
        ),
        (
            "nozzle_volume_type",
            crate::OptionValueKind::Enums,
            "Standard",
        ),
        (
            "retract_lift_above",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "retract_lift_below",
            crate::OptionValueKind::Floats,
            "0",
        ),
        (
            "retract_lift_enforce",
            crate::OptionValueKind::Enums,
            "All Surfaces",
        ),
        (
            "travel_slope",
            crate::OptionValueKind::Floats,
            "3",
        ),
        (
            "z_hop",
            crate::OptionValueKind::Floats,
            "0.4",
        ),
        (
            "z_hop_types",
            crate::OptionValueKind::Enums,
            "Slope Lift",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
