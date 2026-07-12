use super::super::super::{OptionValueKind, option_definition};

#[test]
fn flow_ratio_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("filament_flow_ratio", OptionValueKind::FloatsNullable, "1"),
        ("print_flow_ratio", OptionValueKind::Float, "1"),
        ("top_solid_infill_flow_ratio", OptionValueKind::Float, "1"),
        ("first_layer_flow_ratio", OptionValueKind::Float, "1"),
        ("inner_wall_flow_ratio", OptionValueKind::Float, "1"),
        ("outer_wall_flow_ratio", OptionValueKind::Float, "1"),
        ("overhang_flow_ratio", OptionValueKind::Float, "1"),
        ("set_other_flow_ratios", OptionValueKind::Bool, "false"),
        ("sparse_infill_flow_ratio", OptionValueKind::Float, "1"),
        ("support_flow_ratio", OptionValueKind::Float, "1"),
        ("support_interface_flow_ratio", OptionValueKind::Float, "1"),
        (
            "internal_solid_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
        ),
        ("gap_fill_flow_ratio", OptionValueKind::Float, "1"),
        (
            "bottom_solid_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
        ),
        ("bridge_angle", OptionValueKind::Float, "0"),
        ("internal_bridge_angle", OptionValueKind::Float, "0"),
        ("bridge_density", OptionValueKind::Percent, "100"),
        ("internal_bridge_density", OptionValueKind::Percent, "100"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
