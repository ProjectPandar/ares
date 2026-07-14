use std::collections::BTreeSet;

use super::{rule, source_names};
use crate::options::typed_legacy::{
    Comparison, EmptyValueAction, JsonArrayAllowance, JsonDerivedEffect, LegacyAction,
    RecursionContract, Replacement, StringAllowance, VectorType, WireContract, EXPLICIT_RULES,
    OBSOLETE_INPUTS,
};

mod expected;

#[test]
fn typed_legacy_inventory_has_exact_source_shape() {
    assert_eq!(EXPLICIT_RULES.len(), 76);
    assert_eq!(source_names().len(), 76);
    assert_eq!(OBSOLETE_INPUTS.len(), 44);
    assert_eq!(OBSOLETE_INPUTS, expected::OBSOLETE);
    assert_eq!(OBSOLETE_INPUTS.iter().copied().collect::<BTreeSet<_>>().len(), 44);
    assert!(source_names().is_disjoint(&OBSOLETE_INPUTS.iter().copied().collect()));
    assert_eq!(
        EXPLICIT_RULES
            .iter()
            .filter(|rule| matches!(rule.action, LegacyAction::DeferredProfileBookkeeping { .. }))
            .count(),
        4
    );
    assert_eq!(
        EXPLICIT_RULES
            .iter()
            .filter(|rule| !matches!(rule.action, LegacyAction::DeferredProfileBookkeeping { .. }))
            .count(),
        72
    );
}

#[test]
fn typed_legacy_inventory_has_exact_vector_wire_contracts() {
    let vectors = [
        ("bridge_fan_speed", VectorType::Ints),
        ("chamber_temperatures", VectorType::Ints),
        ("cooling", VectorType::Bools),
        ("default_nozzle_volume_type", VectorType::Enums),
        ("extruder_type", VectorType::Enums),
        ("extruder_variant_list", VectorType::Strings),
        ("filament_extruder_variant", VectorType::Strings),
        ("filament_type", VectorType::Strings),
        ("nozzle_volume_type", VectorType::Enums),
        ("overhang_fan_threshold", VectorType::Enums),
        ("print_extruder_variant", VectorType::Strings),
        ("printer_extruder_variant", VectorType::Strings),
    ];
    for (source, vector) in vectors {
        assert_eq!(rule(source).wire.json_array, JsonArrayAllowance::Flatten(vector));
        assert_eq!(rule(source).wire.vector, Some(vector));
    }
    assert_eq!(
        EXPLICIT_RULES.iter().filter(|rule| rule.wire.vector.is_some()).count(),
        12
    );
    assert_eq!(
        rule("prime_tower_rib_wall").wire.json_array,
        JsonArrayAllowance::ConsumeFirstPass
    );
}

#[test]
fn typed_legacy_inventory_has_exact_string_and_empty_first_pass_contracts() {
    for item in EXPLICIT_RULES {
        let deferred = matches!(item.action, LegacyAction::DeferredProfileBookkeeping { .. });
        let expected = if deferred { StringAllowance::Deferred } else { StringAllowance::Execute };
        assert_eq!(item.wire.json_string, expected, "{} JSON string", item.source);
        assert_eq!(item.wire.xml_string, expected, "{} XML string", item.source);
        assert_eq!(item.recursion, if item.source == "different_settings_to_system" {
            RecursionContract::RecursiveBookkeeping
        } else {
            RecursionContract::SinglePass
        });
        let empty = match item.action {
            LegacyAction::Rename { target }
            | LegacyAction::FeatureFilament { target, .. }
            | LegacyAction::Rewrite { target, .. }
            | LegacyAction::WallOrder { target, .. }
            | LegacyAction::ReplaceAll { target, .. }
            | LegacyAction::FilamentTokenRebuild { target, .. } => {
                EmptyValueAction::Retain { target, value: "" }
            }
            LegacyAction::ConsumeIfContains { .. } => {
                EmptyValueAction::Retain { target: item.source, value: "" }
            }
            LegacyAction::TopOneWall { target, replacement, .. } => {
                EmptyValueAction::Retain { target, value: replacement }
            }
            LegacyAction::PrimeTowerRib { .. } => EmptyValueAction::Consume,
            LegacyAction::DeferredProfileBookkeeping { .. } => EmptyValueAction::Deferred,
        };
        assert_eq!(item.wire.empty_first_pass, empty, "{} empty first pass", item.source);
        let array = match (item.wire.vector, item.wire.empty_first_pass) {
            (Some(vector), _) => JsonArrayAllowance::Flatten(vector),
            (None, EmptyValueAction::Consume) => JsonArrayAllowance::ConsumeFirstPass,
            (None, EmptyValueAction::Deferred) => JsonArrayAllowance::Deferred,
            (None, EmptyValueAction::Retain { .. }) => JsonArrayAllowance::RejectAfterFirstPass,
        };
        assert_eq!(item.wire.json_array, array, "{} JSON array", item.source);
    }

    assert_eq!(
        rule("top_one_wall_type").wire.empty_first_pass,
        EmptyValueAction::Retain { target: "only_one_wall_top", value: "1" }
    );
    assert_eq!(rule("prime_tower_rib_wall").wire.empty_first_pass, EmptyValueAction::Consume);
    for source in [
        "compatible_printers_condition_cummulative",
        "compatible_prints_condition_cummulative",
        "different_settings_to_system",
        "inherits_cummulative",
    ] {
        assert_eq!(rule(source).wire.empty_first_pass, EmptyValueAction::Deferred);
        assert_eq!(rule(source).wire, WireContract::deferred());
    }
    for source in ["rotate_solid_infill_direction", "wall_infill_order"] {
        let target = if source == "rotate_solid_infill_direction" {
            "solid_infill_rotate_template"
        } else {
            "wall_sequence"
        };
        assert_eq!(
            rule(source).wire.empty_first_pass,
            EmptyValueAction::Retain { target, value: "" }
        );
    }
}

#[test]
fn typed_legacy_conditional_actions_have_exact_parameters() {
    for source in [
        "initial_layer_print_height",
        "initial_layer_speed",
        "internal_solid_infill_speed",
        "top_surface_speed",
        "support_interface_speed",
        "outer_wall_speed",
        "support_object_xy_distance",
    ] {
        assert_eq!(rule(source).action, LegacyAction::ConsumeIfContains { needle: "%" });
    }
    assert_eq!(
        rule("top_one_wall_type").action,
        LegacyAction::TopOneWall { target: "only_one_wall_top", consume: "none", replacement: "1" }
    );
    assert_eq!(
        rule("prime_tower_rib_wall").action,
        LegacyAction::PrimeTowerRib { target: "wipe_tower_wall_type", trigger: "1", replacement: "rib" }
    );
}

#[test]
fn typed_legacy_rewrite_actions_have_exact_parameters() {
    let exact = [
        ("curr_bed_type", "curr_bed_type", &[("SuperTack Plate", "Supertack Plate")][..]),
        ("timelapse_type", "timelapse_type", &[("2", "0")][..]),
        ("support_type", "support_type", &[("normal", "normal(manual)"), ("tree", "tree(manual)"), ("hybrid(auto)", "tree(auto)")][..]),
        ("support_base_pattern", "support_base_pattern", &[("none", "hollow")][..]),
        ("overhang_fan_threshold", "overhang_fan_threshold", &[("5%", "10%")][..]),
        ("ensure_vertical_shell_thickness", "ensure_vertical_shell_thickness", &[("1", "ensure_all"), ("0", "ensure_moderate")][..]),
        ("rotate_solid_infill_direction", "solid_infill_rotate_template", &[("1", "0,90"), ("0", "0")][..]),
        ("draft_shield", "draft_shield", &[("limited", "disabled")][..]),
        ("filament_map_mode", "filament_map_mode", &[("Auto", "Auto For Flush")][..]),
        ("wall_direction", "wall_direction", &[("auto", "ccw")][..]),
    ];
    for (source, target, pairs) in exact {
        let LegacyAction::Rewrite { target: actual, comparison, replacements } = rule(source).action
        else { panic!("unexpected action for {source}") };
        assert_eq!((actual, comparison), (target, Comparison::Exact));
        assert_eq!(
            replacements.iter().map(|item| (item.from, item.to)).collect::<Vec<_>>(),
            pairs
        );
    }
    assert_eq!(
        rule("enable_power_loss_recovery").action,
        LegacyAction::Rewrite {
            target: "enable_power_loss_recovery",
            comparison: Comparison::AsciiCaseInsensitive,
            replacements: &[
                Replacement { from: "true", to: "enable" },
                Replacement { from: "1", to: "enable" },
                Replacement { from: "false", to: "disable" },
                Replacement { from: "0", to: "disable" },
            ],
        }
    );
    assert_eq!(
        rule("ironing_angle").action,
        LegacyAction::Rewrite {
            target: "ironing_angle",
            comparison: Comparison::Leading,
            replacements: &[Replacement { from: "-", to: "0" }],
        }
    );
}

#[test]
fn typed_legacy_bulk_and_deferred_actions_have_exact_parameters() {
    for source in [
        "nozzle_volume_type",
        "default_nozzle_volume_type",
        "printer_extruder_variant",
        "print_extruder_variant",
        "filament_extruder_variant",
        "extruder_variant_list",
    ] {
        assert_eq!(
            rule(source).action,
            LegacyAction::ReplaceAll {
                target: source,
                replacements: &[
                    Replacement { from: "Normal", to: "Standard" },
                    Replacement { from: "Big Traffic", to: "High Flow" },
                ],
            }
        );
    }
    assert_eq!(
        rule("extruder_type").action,
        LegacyAction::ReplaceAll {
            target: "extruder_type",
            replacements: &[Replacement { from: "DirectDrive", to: "Direct Drive" }],
        }
    );
    for source in [
        "sparse_infill_pattern",
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
        "ironing_pattern",
        "support_ironing_pattern",
    ] {
        assert_eq!(
            rule(source).action,
            LegacyAction::Rewrite {
                target: source,
                comparison: Comparison::Exact,
                replacements: &[Replacement { from: "zig-zag", to: "rectilinear" }],
            }
        );
    }
    assert_eq!(
        rule("filament_type").action,
        LegacyAction::FilamentTokenRebuild { target: "filament_type", from: "ASA-Aero", to: "ASA-AERO" }
    );
    for (source, target, recursive) in [
        ("inherits_cummulative", Some("inherits_group"), false),
        ("compatible_printers_condition_cummulative", Some("compatible_machine_expression_group"), false),
        ("compatible_prints_condition_cummulative", Some("compatible_process_expression_group"), false),
        ("different_settings_to_system", None, true),
    ] {
        assert_eq!(
            rule(source).action,
            LegacyAction::DeferredProfileBookkeeping { target, recursive }
        );
    }
}

#[test]
fn typed_legacy_json_side_effects_are_exact_and_source_bounded() {
    assert_eq!(
        rule("wall_infill_order").action,
        LegacyAction::WallOrder {
            target: "wall_sequence",
            replacements: &[
                Replacement { from: "inner wall/outer wall/infill", to: "inner wall/outer wall" },
                Replacement { from: "infill/inner wall/outer wall", to: "inner wall/outer wall" },
                Replacement { from: "outer wall/inner wall/infill", to: "outer wall/inner wall" },
                Replacement { from: "infill/outer wall/inner wall", to: "outer wall/inner wall" },
                Replacement { from: "inner-outer-inner wall/infill", to: "inner-outer-inner wall" },
            ],
        }
    );
    assert_eq!(
        rule("support_type").json_effect,
        Some(JsonDerivedEffect {
            triggers: &["hybrid(auto)"],
            target: "support_style",
            value: "tree_hybrid",
        })
    );
    assert_eq!(
        rule("wall_infill_order").json_effect,
        Some(JsonDerivedEffect {
            triggers: &["infill/outer wall/inner wall", "infill/inner wall/outer wall"],
            target: "is_infill_first",
            value: "true",
        })
    );
    assert_eq!(EXPLICIT_RULES.iter().filter(|rule| rule.json_effect.is_some()).count(), 2);
}
