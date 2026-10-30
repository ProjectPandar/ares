//! Shared builders of the arrangement cases, after the helpers of OrcaSlicer's
//! `tests/libslic3r/test_arrange.cpp` at `345efec64`.

use ares_core::arrangement::{
    ArrangeConfig, ArrangeParams, ArrangePolygon, Bed, BoundingBox, Coord, ExPolygon, Point,
    Polygon,
};

/// One millimetre in scaled coordinates (`SCALING_FACTOR` 1e-6).
pub const MM: Coord = 1_000_000;

/// A square of the given scaled side, lower-left at the origin, on bed 0 (`make_square`).
pub fn square(side: Coord) -> ArrangePolygon {
    ArrangePolygon {
        poly: ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(side, 0),
                Point::new(side, side),
                Point::new(0, side),
            ]),
            Vec::new(),
        ),
        bed_idx: 0,
        ..ArrangePolygon::default()
    }
}

/// `count` squares of `side` mm and `height` mm (`squares`).
pub fn squares(count: usize, side: Coord, height: f64) -> Vec<ArrangePolygon> {
    vec![
        ArrangePolygon {
            height,
            ..square(side * MM)
        };
        count
    ]
}

/// The bed `[0, 0]..[width, depth]` mm (`bed`).
pub fn bed(width: Coord, depth: Coord) -> Bed {
    Bed::Box(BoundingBox::new(
        Point::new(0, 0),
        Point::new(width * MM, depth * MM),
    ))
}

/// Default parameters with `min_obj_distance` (`quiet_params`).
pub fn params(min_obj_distance: Coord) -> ArrangeParams {
    ArrangeParams {
        min_obj_distance,
        ..ArrangeParams::default()
    }
}

/// The 200 x 200 mm printable area without a skirt (`bed_config`).
pub fn bed_config() -> ArrangeConfig {
    ArrangeConfig {
        bed_shape: vec![
            Point::new(0, 0),
            Point::new(200 * MM, 0),
            Point::new(200 * MM, 200 * MM),
            Point::new(0, 200 * MM),
        ],
        real_skirt_distance: 0.0,
    }
}

/// The placed silhouette's bounding box.
pub fn placed(item: &ArrangePolygon) -> BoundingBox {
    BoundingBox::from_expolygon(&item.transformed_poly()).unwrap()
}

pub fn inside(outer: BoundingBox, inner: BoundingBox) -> bool {
    inner.min().x() >= outer.min().x()
        && inner.min().y() >= outer.min().y()
        && inner.max().x() <= outer.max().x()
        && inner.max().y() <= outer.max().y()
}

/// The overlapping area of two axis-aligned boxes; touching edges overlap by nothing.
pub fn overlap(a: BoundingBox, b: BoundingBox) -> i128 {
    let width = (a.max().x().min(b.max().x()) - a.min().x().max(b.min().x())).max(0);
    let depth = (a.max().y().min(b.max().y()) - a.min().y().max(b.min().y())).max(0);
    i128::from(width) * i128::from(depth)
}

/// No two placed squares share area (`require_no_overlap`; the cases arrange unrotated squares,
/// which are their own bounding boxes).
pub fn assert_disjoint(items: &[ArrangePolygon]) {
    for (index, a) in items.iter().enumerate() {
        for b in &items[index + 1..] {
            assert_eq!(
                overlap(placed(a), placed(b)),
                0,
                "{:?} overlaps {:?}",
                placed(a),
                placed(b)
            );
        }
    }
}
