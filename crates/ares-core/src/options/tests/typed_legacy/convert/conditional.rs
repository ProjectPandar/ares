use super::{assert_assign, scalar};
use crate::options::typed_legacy::{
    EXPLICIT_RULES, LegacyAction, LegacyOutcome, OBSOLETE_INPUTS, transform_obsolete,
};

#[test]
fn direct_and_feature_rules_produce_canonical_lexical_assignments() {
    assert_assign(scalar("enable_wipe_tower", "1"), "enable_prime_tower", "1");
    assert_assign(
        scalar("ironing_direction", "-45"),
        "ironing_angle",
        "-45",
    );
    assert_assign(
        scalar("infill_extruder", "1"),
        "sparse_infill_filament_id",
        "0",
    );
    assert_assign(
        scalar("infill_extruder", "2"),
        "sparse_infill_filament_id",
        "2",
    );
}

#[test]
fn percentage_rules_consume_only_values_containing_percent() {
    let sources = [
        "initial_layer_print_height",
        "initial_layer_speed",
        "internal_solid_infill_speed",
        "top_surface_speed",
        "support_interface_speed",
        "outer_wall_speed",
        "support_object_xy_distance",
    ];

    for source in sources {
        assert_eq!(scalar(source, "75%"), LegacyOutcome::Consume, "{source}");
        assert_assign(scalar(source, "75"), source, "75");
    }
}

#[test]
fn top_one_wall_and_prime_tower_rib_cover_both_conditional_branches() {
    assert_eq!(
        scalar("top_one_wall_type", "none"),
        LegacyOutcome::Consume
    );
    assert_assign(
        scalar("top_one_wall_type", "top"),
        "only_one_wall_top",
        "1",
    );
    assert_assign(
        scalar("prime_tower_rib_wall", "1"),
        "wipe_tower_wall_type",
        "rib",
    );
    assert_eq!(
        scalar("prime_tower_rib_wall", "0"),
        LegacyOutcome::Consume
    );
}

#[test]
fn every_obsolete_input_is_consumed_and_other_names_are_not_claimed() {
    for source in OBSOLETE_INPUTS {
        assert_eq!(
            transform_obsolete(source),
            Some(LegacyOutcome::Consume),
            "{source}"
        );
    }
    assert_eq!(transform_obsolete("not_obsolete"), None);
}

#[test]
fn profile_bookkeeping_rules_return_deferred_outcomes() {
    for rule in EXPLICIT_RULES {
        if let LegacyAction::DeferredProfileBookkeeping { target, recursive } = rule.action {
            assert_eq!(
                scalar(rule.source, "ignored"),
                LegacyOutcome::Deferred {
                    source: rule.source,
                    target,
                    recursive,
                }
            );
        }
    }
}
