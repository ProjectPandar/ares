//! Arrangement parameter updates and bed-exclusion rules, ported from OrcaSlicer's
//! `tests/libslic3r/test_arrange.cpp` at v2.5.0-beta `345efec64`, through the public
//! `arrangement` API.

use crate::support::{MM, bed_config, square, squares};
use ares_core::arrangement::{
    ArrangeParams, ArrangePolygon, bed_exclusion_applies, has_bed_exclusion_regions,
    update_selected_items_inflation,
};

/// The sequential-print floor compares object height with the nozzle (`seq_print_params`).
const NOZZLE_HEIGHT: f64 = 2.5;

fn seq_print_params(min_obj_distance: i64, object_skirt_offset: f32) -> ArrangeParams {
    ArrangeParams {
        min_obj_distance,
        is_seq_print: true,
        clearance_radius: 30.0,
        nozzle_height: NOZZLE_HEIGHT as f32,
        object_skirt_offset,
        ..ArrangeParams::default()
    }
}

/// Two 20 mm squares of the given heights after `update_selected_items_inflation`.
fn floored(
    first_height: f64,
    second_height: f64,
    params: &mut ArrangeParams,
) -> Vec<ArrangePolygon> {
    let mut items = vec![
        squares(1, 20, first_height).remove(0),
        squares(1, 20, second_height).remove(0),
    ];
    update_selected_items_inflation(&mut items, &bed_config(), params);
    items
}

/// The floor is `scaled(max + 0.001)`, at most 0.01 mm over the expected value, and each item
/// takes half of it.
fn assert_floor(params: &ArrangeParams, items: &[ArrangePolygon], expected_mm: i64) {
    assert!(
        params.min_obj_distance >= expected_mm * MM,
        "{}",
        params.min_obj_distance
    );
    assert!(
        params.min_obj_distance <= expected_mm * MM + MM / 100,
        "{}",
        params.min_obj_distance
    );
    assert_eq!(items[0].inflation, params.min_obj_distance / 2);
}

#[test]
fn sequential_print_needs_the_full_clearance_between_objects_taller_than_the_nozzle() {
    let mut params = seq_print_params(0, 0.0);
    let items = floored(5.0, 5.0, &mut params);
    assert_floor(&params, &items, 30);
}

#[test]
fn sequential_print_counts_an_object_at_the_nozzle_height_as_tall() {
    let mut params = seq_print_params(0, 0.0);
    let items = floored(2.5, 2.5, &mut params);
    assert_floor(&params, &items, 30);
}

#[test]
fn sequential_print_needs_the_full_clearance_for_one_tall_object_among_short_ones() {
    let mut params = seq_print_params(0, 0.0);
    let items = floored(1.25, 5.0, &mut params);
    assert_floor(&params, &items, 30);
}

/// Full `MAX_OUTER_NOZZLE_DIAMETER` (4 mm), `40ecc66f7` `Arrange.cpp:169`.
#[test]
fn sequential_print_keeps_only_the_nozzle_width_floor_for_objects_the_nozzle_clears() {
    let mut params = seq_print_params(0, 0.0);
    let items = floored(1.25, 1.25, &mut params);
    assert_floor(&params, &items, 4);
}

#[test]
fn sequential_print_raises_the_floor_for_short_objects_by_a_wide_skirt() {
    let mut params = seq_print_params(0, 3.0);
    let items = floored(1.25, 1.25, &mut params);
    assert_floor(&params, &items, 6);
}

#[test]
fn sequential_print_keeps_an_object_distance_already_above_the_floor() {
    let mut params = seq_print_params(60 * MM, 0.0);
    floored(5.0, 5.0, &mut params);
    assert_eq!(params.min_obj_distance, 60 * MM);
}

#[test]
fn layered_printing_does_not_floor_the_object_distance() {
    let mut params = ArrangeParams {
        is_seq_print: false,
        ..seq_print_params(0, 0.0)
    };
    floored(5.0, 5.0, &mut params);
    assert_eq!(params.min_obj_distance, 0);
}

#[test]
fn bed_exclusions_apply_by_physical_nozzle_and_overlapping_z_range() {
    let object = ArrangePolygon {
        has_z_range: true,
        z_min: 0.0,
        z_max: 20.0,
        bed_exclusion_extruder_ids: vec![1],
        ..square(20 * MM)
    };
    let exclusion = ArrangePolygon {
        is_bed_exclusion: true,
        has_z_range: true,
        z_min: 10.0,
        z_max: 30.0,
        bed_exclusion_extruder_id: 1,
        ..square(20 * MM)
    };
    assert!(bed_exclusion_applies(&object, &exclusion));

    let other_nozzle = ArrangePolygon {
        bed_exclusion_extruder_id: 0,
        ..exclusion
    };
    assert!(!bed_exclusion_applies(&object, &other_nozzle));
    // Empty physical-nozzle assignments are unresolved, so arrangement stays conservative.
    let unresolved = ArrangePolygon {
        bed_exclusion_extruder_ids: Vec::new(),
        ..object.clone()
    };
    assert!(bed_exclusion_applies(&unresolved, &other_nozzle));
    let below = ArrangePolygon {
        bed_exclusion_extruder_ids: vec![0],
        z_max: 9.0,
        ..object.clone()
    };
    assert!(!bed_exclusion_applies(&below, &other_nozzle));
    // Touching at one Z plane is a collision.
    let touching = ArrangePolygon {
        bed_exclusion_extruder_ids: vec![0],
        z_max: 10.0,
        ..object
    };
    assert!(bed_exclusion_applies(&touching, &other_nozzle));
}

#[test]
fn shared_bed_exclusions_apply_to_every_arranged_object() {
    let object = ArrangePolygon {
        bed_exclusion_extruder_ids: vec![1],
        ..square(20 * MM)
    };
    let shared = ArrangePolygon {
        is_bed_exclusion: true,
        bed_exclusion_extruder_id: -1,
        ..square(20 * MM)
    };
    assert!(has_bed_exclusion_regions(std::slice::from_ref(&shared)));
    assert!(!has_bed_exclusion_regions(std::slice::from_ref(&object)));
    assert!(bed_exclusion_applies(&object, &shared));
}
