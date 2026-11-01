//! The IDEX/IQEX arranger, ported from OrcaSlicer's `tests/libslic3r/test_imex_arrange.cpp` at
//! v2.5.0-beta `345efec64`, through the public `arrangement::imex` API. The GUI's ArrangeJob is
//! its only caller, so these upstream cases are the oracle (`web-plater-arrange-imex.md`).

use ares_core::arrangement::imex::{
    BoundingBoxf, ImexArrangeInput, ImexArrangeZones, ImexArranger,
};
use ares_core::arrangement::{
    ArrangeParams, ArrangePolygon, Bed, BoundingBox, Coord, ExPolygon, Point, Polygon,
    SCALING_FACTOR, UNARRANGED, arrange as nest,
};

// Fixtures, `test_imex_arrange.cpp:21-96`. A 300 x 200 mm bed; a two-carriage IDEX in a parallel
// mode gives T0 the left half, and a mirror mode adds a 30 mm strip along its inner edge.
const LEFT_HALF: BoundingBoxf = BoundingBoxf {
    min: [0., 0.],
    max: [150., 200.],
};
const RIGHT_HALF: BoundingBoxf = BoundingBoxf {
    min: [150., 0.],
    max: [300., 200.],
};
const STRIP: BoundingBoxf = BoundingBoxf {
    min: [120., 0.],
    max: [150., 200.],
};

fn copy() -> ImexArrangeZones {
    ImexArrangeZones {
        primary_zone: LEFT_HALF,
        collision_zones: Vec::new(),
    }
}

fn mirror() -> ImexArrangeZones {
    ImexArrangeZones {
        primary_zone: LEFT_HALF,
        collision_zones: vec![STRIP],
    }
}

/// `scaled<coord_t>(double)`: divided and truncated.
fn scaled(value: f64) -> Coord {
    (value / SCALING_FACTOR) as Coord
}

fn scaled_point([x, y]: [f64; 2]) -> Point {
    Point::new(scaled(x), scaled(y))
}

/// `BoundingBox::polygon` of the scaled box: min, (max.x, min.y), max, (min.x, max.y).
fn scaled_box_points(min: [f64; 2], max: [f64; 2]) -> Vec<Point> {
    vec![
        scaled_point(min),
        scaled_point([max[0], min[1]]),
        scaled_point(max),
        scaled_point([min[0], max[1]]),
    ]
}

/// A 1 mm bed edge margin, and no rotation.
fn test_params() -> ArrangeParams {
    ArrangeParams {
        bed_shrink_x: 1.,
        bed_shrink_y: 1.,
        allow_rotations: false,
        ..ArrangeParams::default()
    }
}

/// The bed shrunk by that 1 mm margin, as `get_shrink_bedpts` hands it to the arranger.
fn shrunk_bed() -> Vec<Point> {
    scaled_box_points([1., 1.], [299., 199.])
}

/// A `width` x `depth` mm part, kept 1 mm from its neighbours, on bed 0.
fn part(width: f64, depth: f64) -> ArrangePolygon {
    let contour = scaled_box_points([-width / 2., -depth / 2.], [width / 2., depth / 2.]);
    ArrangePolygon {
        poly: ExPolygon::new(Polygon::new(contour), Vec::new()),
        inflation: scaled(1.),
        extrude_ids: vec![0],
        height: 10.,
        bed_idx: 0,
        ..ArrangePolygon::default()
    }
}

fn squares(count: i32, size: f64) -> Vec<ArrangePolygon> {
    (0..count)
        .map(|itemid| ArrangePolygon {
            itemid,
            ..part(size, size)
        })
        .collect()
}

/// Arranges `items` the way ArrangeJob does.
fn arrange(items: &mut [ArrangePolygon], input: &ImexArrangeInput) {
    let params = test_params();
    let imex = ImexArranger::new(input, &params, &shrunk_bed());
    let mut fixed = Vec::new();
    imex.add_keep_outs(&mut fixed);
    let before = items.to_vec();
    nest(items, &fixed, &Bed::from_points(&imex.bed_shape()), &params);
    imex.finish(items, &before, &fixed, &params);
}

fn scaled_bounds(points: &[Point]) -> BoundingBox {
    BoundingBox::from_expolygon(&ExPolygon::new(Polygon::new(points.to_vec()), Vec::new())).unwrap()
}

fn placed(item: &ArrangePolygon) -> BoundingBox {
    BoundingBox::from_expolygon(&item.transformed_poly()).unwrap()
}

/// The union box of the placed parts, in mm.
fn extents(items: &[ArrangePolygon]) -> BoundingBoxf {
    let (mut min, mut max) = ([Coord::MAX; 2], [Coord::MIN; 2]);
    for item in items {
        let bounds = placed(item);
        min = [min[0].min(bounds.min().x()), min[1].min(bounds.min().y())];
        max = [max[0].max(bounds.max().x()), max[1].max(bounds.max().y())];
    }
    let unscaled = |p: [Coord; 2]| p.map(|v| v as f64 * SCALING_FACTOR);
    BoundingBoxf {
        min: unscaled(min),
        max: unscaled(max),
    }
}

/// `BoundingBoxBase::contains`: inclusive on every side.
fn contains(outer: BoundingBoxf, [x, y]: [f64; 2]) -> bool {
    x >= outer.min[0] && x <= outer.max[0] && y >= outer.min[1] && y <= outer.max[1]
}

fn center(box_: BoundingBoxf) -> [f64; 2] {
    [
        (box_.min[0] + box_.max[0]) / 2.,
        (box_.min[1] + box_.max[1]) / 2.,
    ]
}

/// `WithinAbs(target, margin)`.
fn within(value: f64, target: f64, margin: f64) -> bool {
    (value - target).abs() <= margin
}

/// `crosses` (`:91-94`) intersects the placed contour with the box. Every part here is an
/// unrotated rectangle and every box axis-aligned, so that is non-empty exactly when the two
/// boxes overlap with positive area.
fn crosses(item: &ArrangePolygon, box_: BoundingBoxf) -> bool {
    let (part, zone) = (
        placed(item),
        scaled_bounds(&scaled_box_points(box_.min, box_.max)),
    );
    part.min().x() < zone.max().x()
        && zone.min().x() < part.max().x()
        && part.min().y() < zone.max().y()
        && zone.min().y() < part.max().y()
}

#[test]
fn imex_arranger_changes_nothing_without_zones() {
    // `:98-110`
    let input = ImexArrangeInput {
        beds: vec![None, None],
        adds_plates: true,
        ..ImexArrangeInput::default()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert!(!imex.active());
    assert_eq!(imex.bed_shape(), shrunk_bed());
    let mut fixed = Vec::new();
    imex.add_keep_outs(&mut fixed);
    assert!(fixed.is_empty());
}

#[test]
fn imex_plates_sharing_a_zone_arrange_inside_it_clear_of_the_strip() {
    // `:112-135`: the second plate computes the same zone off in the last bits.
    let mut shifted = mirror();
    shifted.primary_zone.max[0] += 1e-9;
    let input = ImexArrangeInput {
        beds: vec![Some(mirror()), Some(shifted)],
        adds_plates: true,
        added_plate: Some(mirror()),
        ..ImexArrangeInput::default()
    };
    // The bed shape is the zone less the bed's 1 mm edge margin.
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    let shape = scaled_bounds(&imex.bed_shape());
    assert_eq!(shape.min(), scaled_point([1., 1.]));
    assert_eq!(shape.max(), scaled_point([149., 199.]));
    // Enough parts that, centred in the zone, they would reach the strip.
    let mut items = squares(20, 20.);
    arrange(&mut items, &input);
    for item in &items {
        assert!(item.bed_idx >= 0);
        assert!(!crosses(item, STRIP));
    }
}

#[test]
fn imex_parallel_plate_beside_a_primary_plate_is_centred_in_its_zone() {
    // `:137-157`: different zones keep the whole bed as the bed shape and fence the mirror plate.
    let input = ImexArrangeInput {
        beds: vec![Some(mirror()), None],
        adds_plates: true,
        ..ImexArrangeInput::default()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert_eq!(imex.bed_shape(), shrunk_bed());
    let mut items = squares(4, 20.);
    arrange(&mut items, &input);
    assert!(items.iter().all(|item| item.bed_idx == 0));
    let pile = extents(&items);
    assert!(contains(LEFT_HALF, pile.min));
    assert!(contains(LEFT_HALF, pile.max));
    // Centred in the zone, rather than piled against its edge nearest the bed's centre.
    assert!(within(center(pile)[0], 75., 5.));
    assert!(within(center(pile)[1], 100., 5.));
}

#[test]
fn imex_primary_plate_beside_a_parallel_plate_uses_the_whole_bed() {
    // `:159-170`
    let input = ImexArrangeInput {
        beds: vec![None, Some(mirror())],
        adds_plates: true,
        ..ImexArrangeInput::default()
    };
    let mut items = squares(4, 20.);
    arrange(&mut items, &input);
    assert!(items.iter().all(|item| item.bed_idx == 0));
    assert!(within(center(extents(&items))[0], 150., 5.));
}

#[test]
fn imex_part_too_big_for_its_zone_is_left_unarranged() {
    // `:172-189`, one section each: arranging one plate the zone is the bed shape; arranging
    // all plates it is fenced in.
    for input in [
        ImexArrangeInput {
            beds: vec![Some(copy())],
            ..ImexArrangeInput::default()
        },
        ImexArrangeInput {
            beds: vec![Some(copy()), None],
            adds_plates: true,
            ..ImexArrangeInput::default()
        },
    ] {
        // Narrower than the bed but wider than the zone.
        let mut items = vec![part(180., 50.)];
        arrange(&mut items, &input);
        assert_eq!(items[0].bed_idx, UNARRANGED);
    }
}

#[test]
fn imex_strip_keeps_room_for_the_bed_margin_and_the_brim() {
    // `:191-205`
    let input = ImexArrangeInput {
        beds: vec![Some(mirror())],
        brim: 5.,
        ..ImexArrangeInput::default()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    let mut fixed = Vec::new();
    imex.add_keep_outs(&mut fixed);
    assert_eq!(fixed.len(), 1);
    // Grown by the 1 mm margin and the 5 mm brim on every side.
    let strip = BoundingBox::from_expolygon(&fixed[0].poly).unwrap();
    assert_eq!(strip.min(), scaled_point([114., -6.]));
    assert_eq!(strip.max(), scaled_point([156., 206.]));
}

#[test]
fn imex_added_plate_takes_the_zones_it_will_have() {
    // `:207-248`, one block per section from the same setup.
    let base = ImexArrangeInput {
        beds: vec![Some(copy())],
        adds_plates: true,
        ..ImexArrangeInput::default()
    };
    let on = |fixed: &[ArrangePolygon], bed_idx: i32| {
        fixed.iter().filter(|item| item.bed_idx == bed_idx).count()
    };

    // The same zone keeps it the bed shape, with the new plates' strips.
    let input = ImexArrangeInput {
        added_plate: Some(mirror()),
        ..base.clone()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert_ne!(imex.bed_shape(), shrunk_bed());
    let mut fixed = Vec::new();
    imex.add_keep_outs(&mut fixed);
    assert_eq!(on(&fixed, 0), 0);
    assert_eq!(on(&fixed, 1), 1);

    // A new plate in Primary gets the whole bed, so the zone is fenced in instead.
    let imex = ImexArranger::new(&base, &test_params(), &shrunk_bed());
    assert_eq!(imex.bed_shape(), shrunk_bed());
    let mut fixed = Vec::new();
    imex.add_keep_outs(&mut fixed);
    assert!(fixed.iter().all(|item| item.bed_idx == 0));

    // Another zone makes the whole bed the bed shape.
    let right_half = ImexArrangeZones {
        primary_zone: RIGHT_HALF,
        collision_zones: Vec::new(),
    };
    let input = ImexArrangeInput {
        added_plate: Some(right_half.clone()),
        ..base.clone()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert_eq!(imex.bed_shape(), shrunk_bed());

    // Arranging one plate never adds one.
    let input = ImexArrangeInput {
        adds_plates: false,
        added_plate: Some(right_half),
        ..base
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert_ne!(imex.bed_shape(), shrunk_bed());
}

#[test]
fn imex_zone_narrower_than_its_margins_takes_no_parts() {
    // `:250-260`
    let input = ImexArrangeInput {
        beds: vec![Some(ImexArrangeZones {
            primary_zone: BoundingBoxf {
                min: [0., 0.],
                max: [1.5, 200.],
            },
            collision_zones: Vec::new(),
        })],
        ..ImexArrangeInput::default()
    };
    let imex = ImexArranger::new(&input, &test_params(), &shrunk_bed());
    assert_eq!(imex.bed_shape(), shrunk_bed());
    let mut items = squares(1, 20.);
    arrange(&mut items, &input);
    assert_eq!(items[0].bed_idx, UNARRANGED);
}

#[test]
fn imex_parts_spilling_onto_a_new_primary_plate_use_its_whole_bed() {
    // `:263-287`: every plate in copy mode, while a plate the arrange adds is in Primary. The
    // zone holds 3 x 4 of these 40 mm parts, so the rest spill onto a new plate.
    let input = ImexArrangeInput {
        beds: vec![Some(copy())],
        adds_plates: true,
        ..ImexArrangeInput::default()
    };
    let mut items = squares(16, 40.);
    arrange(&mut items, &input);
    let mut spilled = Vec::new();
    for item in &items {
        assert!(item.bed_idx >= 0);
        if item.bed_idx == 0 {
            let placed = extents(std::slice::from_ref(item));
            assert!(contains(LEFT_HALF, placed.min));
            assert!(contains(LEFT_HALF, placed.max));
        } else {
            spilled.push(item.clone());
        }
    }
    assert!(!spilled.is_empty());
    // Centred on the whole bed, not packed into the copy plate's zone.
    assert!(within(center(extents(&spilled))[0], 150., 10.));
}
