//! Arrangement placement, ported from OrcaSlicer's `tests/libslic3r/test_arrange.cpp` and
//! `tests/libnest2d/libnest2d_tests.cpp` at v2.5.0-beta `345efec64`, through the public
//! `arrangement::arrange`.

use crate::support::{
    MM, assert_disjoint, bed, bed_config, inside, overlap, params, placed, square, squares,
};
use ares_core::arrangement::{
    ArrangeParams, ArrangePolygon, Bed, BoundingBox, ExPolygon, Point, Polygon, UNARRANGED,
    arrange, get_shrink_bedpts, update_arrange_params, update_selected_items_inflation,
};

fn bed_box(width: i64, depth: i64) -> BoundingBox {
    BoundingBox::new(Point::new(0, 0), Point::new(width * MM, depth * MM))
}

#[test]
fn arrange_places_every_item_on_the_physical_bed() {
    let mut items = squares(5, 20, 0.0);
    arrange(&mut items, &[], &bed(200, 200), &params(MM));
    assert!(items.iter().all(|item| item.bed_idx == 0));
}

#[test]
fn arranged_items_stay_within_the_bed() {
    let mut items = squares(6, 30, 0.0);
    arrange(&mut items, &[], &bed(200, 200), &params(MM));
    for item in &items {
        assert_eq!(item.bed_idx, 0);
        assert!(inside(bed_box(200, 200), placed(item)));
    }
}

#[test]
fn arranged_items_do_not_overlap() {
    let mut items = squares(6, 40, 0.0);
    arrange(&mut items, &[], &bed(250, 250), &params(2 * MM));
    assert_disjoint(&items);
}

/// Per-item inflation is how the arranger keeps clearance: four squares inflated 4 mm each end
/// at least 8 mm apart (0.5 mm slack for nester rounding).
#[test]
fn arrange_spaces_items_by_their_inflation() {
    let mut items = vec![
        ArrangePolygon {
            inflation: 4 * MM,
            ..square(20 * MM)
        };
        4
    ];
    arrange(&mut items, &[], &bed(200, 200), &params(0));
    for (index, a) in items.iter().enumerate() {
        for b in &items[index + 1..] {
            let (a, b) = (placed(a), placed(b));
            let gap_x = (b.min().x() - a.max().x())
                .max(a.min().x() - b.max().x())
                .max(0) as f64;
            let gap_y = (b.min().y() - a.max().y())
                .max(a.min().y() - b.max().y())
                .max(0) as f64;
            assert!(
                gap_x.hypot(gap_y) >= 7.5 * MM as f64,
                "{a:?} and {b:?} are closer than 8 mm"
            );
        }
    }
}

/// An item that cannot be placed keeps the position it came in with.
#[test]
fn an_item_larger_than_the_bed_keeps_its_input_and_stays_unarranged() {
    let mut items = vec![
        square(20 * MM),
        ArrangePolygon {
            translation: Point::new(10 * MM, 20 * MM),
            ..square(400 * MM)
        },
    ];
    arrange(&mut items, &[], &bed(200, 200), &params(MM));
    assert_eq!(items[0].bed_idx, 0);
    assert_eq!(items[1].bed_idx, UNARRANGED);
    assert_eq!(items[1].translation, Point::new(10 * MM, 20 * MM));
}

/// The arrange bed's margin and the item's brim must not keep an item that fits the printable
/// area from being placed; one larger than it is still refused.
#[test]
fn an_item_filling_the_printable_area_is_arranged() {
    let config = bed_config();
    let mut fitting = vec![ArrangePolygon {
        brim_width: 1.0,
        ..squares(1, 197, 0.0).remove(0)
    }];
    let mut fitting_params = params(0);
    update_arrange_params(&mut fitting_params, &config, &fitting);
    update_selected_items_inflation(&mut fitting, &config, &mut fitting_params);
    arrange(
        &mut fitting,
        &[],
        &Bed::from_points(&get_shrink_bedpts(&config, &fitting_params)),
        &fitting_params,
    );
    assert_eq!(fitting[0].bed_idx, 0);

    let mut large = squares(1, 201, 0.0);
    let mut large_params = params(0);
    update_arrange_params(&mut large_params, &config, &large);
    update_selected_items_inflation(&mut large, &config, &mut large_params);
    arrange(
        &mut large,
        &[],
        &Bed::from_points(&get_shrink_bedpts(&config, &large_params)),
        &large_params,
    );
    assert_eq!(large[0].bed_idx, UNARRANGED);
}

/// Eight 90 mm squares cannot share a 200 x 200 mm bed.
#[test]
fn items_overflowing_one_bed_spill_onto_virtual_beds() {
    let mut items = squares(8, 90, 0.0);
    arrange(&mut items, &[], &bed(200, 200), &params(2 * MM));
    assert!(items.iter().all(|item| item.bed_idx >= 0));
    assert!(items.iter().any(|item| item.bed_idx >= 1));
}

#[test]
fn arrange_handles_an_empty_input() {
    let mut items: Vec<ArrangePolygon> = Vec::new();
    arrange(&mut items, &[], &bed(200, 200), &params(0));
    assert!(items.is_empty());
}

/// `nest()` leaves an empty or a zero-area item untouched (libnest2d_tests.cpp).
#[test]
fn arrange_leaves_an_empty_or_zero_area_item_unarranged() {
    let mut items = vec![
        ArrangePolygon::default(),
        ArrangePolygon {
            poly: ExPolygon::new(Polygon::new(vec![Point::new(0, 200)]), Vec::new()),
            ..ArrangePolygon::default()
        },
    ];
    arrange(&mut items, &[], &bed(250, 210), &params(0));
    assert!(items.iter().all(|item| item.bed_idx == UNARRANGED));
}

/// `do_final_align = false` skips the recentering.
#[test]
fn arrange_without_final_alignment_keeps_items_disjoint() {
    let mut items = squares(6, 40, 0.0);
    arrange(
        &mut items,
        &[],
        &bed(250, 250),
        &ArrangeParams {
            do_final_align: false,
            ..params(2 * MM)
        },
    );
    assert!(items.iter().all(|item| item.bed_idx == 0));
    assert_disjoint(&items);
}

/// `best_object_pos` (0.3, 0.7) centres the pile at (75, 175) mm on a 250 mm bed.
#[test]
fn arrange_centers_a_pile_that_fits_on_the_custom_alignment_point() {
    let mut items = squares(4, 30, 0.0);
    arrange(
        &mut items,
        &[],
        &bed(250, 250),
        &ArrangeParams {
            align_center: [0.3, 0.7],
            ..params(2 * MM)
        },
    );
    assert!(items.iter().all(|item| item.bed_idx == 0));
    let mut pile = placed(&items[0]);
    for item in &items[1..] {
        let next = placed(item);
        pile = BoundingBox::new(
            Point::new(
                pile.min().x().min(next.min().x()),
                pile.min().y().min(next.min().y()),
            ),
            Point::new(
                pile.max().x().max(next.max().x()),
                pile.max().y().max(next.max().y()),
            ),
        );
    }
    assert!((pile.center().x() - 75 * MM).abs() <= MM / 2, "{pile:?}");
    assert!((pile.center().y() - 175 * MM).abs() <= MM / 2, "{pile:?}");
    assert_disjoint(&items);
}

/// Four 40 mm parts centred on the 30% mark of a 200 x 50 mm bed would start 24 mm left of it
/// (#15727); the pile stops at the left edge instead.
#[test]
fn arrange_keeps_a_custom_aligned_pile_within_the_bed() {
    let mut items = squares(4, 40, 0.0);
    arrange(
        &mut items,
        &[],
        &bed(200, 50),
        &ArrangeParams {
            align_center: [0.3, 0.5],
            ..params(2 * MM)
        },
    );
    for item in &items {
        assert_eq!(item.bed_idx, 0);
        assert!(inside(bed_box(200, 50), placed(item)));
    }
    assert!(
        items
            .iter()
            .map(|item| placed(item).min().x())
            .min()
            .unwrap()
            < 10 * MM
    );
    assert_disjoint(&items);
}

/// The final alignment centres the box around the selection and the fixed part, so the check
/// must test where the selection actually lands.
#[test]
fn arrange_keeps_a_selection_on_the_bed_and_clear_of_a_fixed_part() {
    let fixed = vec![ArrangePolygon {
        translation: Point::new(80 * MM, 80 * MM),
        ..square(40 * MM)
    }];
    let mut items = vec![
        ArrangePolygon {
            inflation: MM,
            ..square(40 * MM)
        };
        3
    ];
    arrange(&mut items, &fixed, &bed(200, 200), &params(0));
    for item in &items {
        assert_eq!(item.bed_idx, 0);
        assert!(inside(bed_box(200, 200), placed(item)));
    }
    items.extend(fixed);
    assert_disjoint(&items);
}

/// An off-centre `best_object_pos` (the A1 mini and the H2 family) used to push the pile off the
/// bed (#15727); it stops at the edge it was aimed at.
#[test]
fn arrange_keeps_a_flat_bed_pile_aligned_near_an_edge_on_the_bed() {
    let mut items = squares(4, 90, 0.0);
    arrange(
        &mut items,
        &[],
        &bed(95, 500),
        &ArrangeParams {
            align_center: [0.5, 0.05],
            ..params(2 * MM)
        },
    );
    for item in &items {
        assert_eq!(item.bed_idx, 0);
        assert!(inside(bed_box(95, 500), placed(item)));
    }
    assert!(
        items
            .iter()
            .map(|item| placed(item).min().y())
            .min()
            .unwrap()
            < 10 * MM
    );
    assert_disjoint(&items);
}

#[test]
fn arrange_keeps_relevant_objects_outside_conditional_exclusion_obstacles() {
    let exclusion = ArrangePolygon {
        translation: Point::new(35 * MM, 35 * MM),
        is_bed_exclusion: true,
        bed_exclusion_extruder_id: 0,
        ..square(30 * MM)
    };
    let mut items = vec![
        ArrangePolygon {
            bed_exclusion_extruder_ids: vec![0],
            ..square(20 * MM)
        };
        4
    ];
    arrange(
        &mut items,
        std::slice::from_ref(&exclusion),
        &bed(100, 100),
        &params(MM),
    );
    for item in &items {
        assert_eq!(item.bed_idx, 0);
        assert_eq!(overlap(placed(item), placed(&exclusion)), 0);
    }
    assert_disjoint(&items);
}
