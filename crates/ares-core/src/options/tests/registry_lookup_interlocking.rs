use crate::{OptionValueKind, option_definition};

#[test]
fn inheritance_interlocking_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "calib_flowrate_topinfill_special_order",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "inherits",
            OptionValueKind::String,
            "",
        ),
        (
            "inherits_group",
            OptionValueKind::Strings,
            "",
        ),
        (
            "interface_shells",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "interlocking_beam",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "interlocking_beam_layer_count",
            OptionValueKind::Int,
            "2",
        ),
        (
            "interlocking_beam_width",
            OptionValueKind::Float,
            "0.8",
        ),
        (
            "interlocking_boundary_avoidance",
            OptionValueKind::Int,
            "2",
        ),
        (
            "interlocking_depth",
            OptionValueKind::Int,
            "2",
        ),
        (
            "interlocking_orientation",
            OptionValueKind::Float,
            "22.5",
        ),
        (
            "mmu_segmented_region_interlocking_depth",
            OptionValueKind::Float,
            "0",
        ),
        (
            "mmu_segmented_region_max_width",
            OptionValueKind::Float,
            "0",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
