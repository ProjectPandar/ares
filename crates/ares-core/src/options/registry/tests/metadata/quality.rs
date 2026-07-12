use super::super::super::{OptionValueKind, option_definition};

#[test]
fn one_wall_quality_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        ("precise_outer_wall", OptionValueKind::Bool, "true"),
        ("only_one_wall_top", OptionValueKind::Bool, "false"),
        (
            "min_width_top_surface",
            OptionValueKind::FloatOrPercent,
            "300%",
        ),
        ("only_one_wall_first_layer", OptionValueKind::Bool, "false"),
        ("overhang_reverse", OptionValueKind::Bool, "false"),
        (
            "overhang_reverse_internal_only",
            OptionValueKind::Bool,
            "false",
        ),
        ("counterbore_hole_bridging", OptionValueKind::Enum, "none"),
        (
            "overhang_reverse_threshold",
            OptionValueKind::FloatOrPercent,
            "50%",
        ),
        (
            "extra_perimeters_on_overhangs",
            OptionValueKind::Bool,
            "false",
        ),
        ("bridge_no_support", OptionValueKind::Bool, "false"),
        (
            "dont_filter_internal_bridges",
            OptionValueKind::Enum,
            "disabled",
        ),
        (
            "enable_extra_bridge_layer",
            OptionValueKind::Enum,
            "disabled",
        ),
        ("max_bridge_length", OptionValueKind::Float, "10"),
        (
            "ensure_vertical_shell_thickness",
            OptionValueKind::Enum,
            "ensure_all",
        ),
        (
            "top_surface_pattern",
            OptionValueKind::Enum,
            "monotonicline",
        ),
        ("bottom_surface_pattern", OptionValueKind::Enum, "monotonic"),
        (
            "internal_solid_infill_pattern",
            OptionValueKind::Enum,
            "monotonic",
        ),
        ("small_perimeter_threshold", OptionValueKind::Float, "0"),
        (
            "wall_sequence",
            OptionValueKind::Enum,
            "inner wall/outer wall",
        ),
        ("wall_direction", OptionValueKind::Enum, "ccw"),
        ("thick_bridges", OptionValueKind::Bool, "false"),
        ("thick_internal_bridges", OptionValueKind::Bool, "true"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
