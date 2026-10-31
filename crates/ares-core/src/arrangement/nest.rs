//! The libnest2d subset `Arrange.cpp` instantiates at `345efec64`: `_Item<ExPolygon>`, the
//! convex-only `_NofitPolyPlacer` and `_FirstFitSelection`, over an arena of items that the
//! placers and the selection refer to by index (upstream's `std::reference_wrapper`s).

pub(crate) mod bin;
mod edges;
pub(crate) mod geometry;
pub(crate) mod item;
pub(crate) mod placer;
pub(crate) mod selection;

use geometry::{NBox, distance, shape_box};
use item::Item;

use super::arranger::Arranger;
use super::nlopt::subplex_minimize;
use crate::geometry::{ExPolygon, Point};

/// `NfpPConfig::Alignment`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Alignment {
    Center,
    BottomLeft,
    TopRight,
    DontAlign,
    UserDefined,
}

/// The bin a placer packs into (`to_nestbin`).
#[derive(Clone, Debug)]
pub(crate) enum Bin {
    Box(NBox),
    Circle { center: Point, radius: f64 },
    Polygon(ExPolygon),
}

/// The placer's `object_function`: the arranger's per bin type (`get_objfn`), or after fixed
/// items were preloaded, `fixed_overfit(objfunc(item, starting_point), binbb)`.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ObjectFn {
    Bin { origin_pack: Point },
    Preloaded { binbb: NBox, starting_point: Point },
}

/// The item arena and the arranger whose hooks the placers call.
pub(crate) struct Nest {
    pub(crate) items: Vec<Item>,
    pub(crate) arranger: Arranger,
}

impl Nest {
    /// `before_packing`.
    pub(crate) fn before_packing(
        &mut self,
        pile: &[ExPolygon],
        packed: &[usize],
        remaining: &[usize],
    ) {
        self.arranger
            .before_packing(&self.items, pile, packed, remaining);
    }

    pub(crate) fn objective(&self, object_fn: &ObjectFn, item: &Item) -> f64 {
        self.arranger.objective(&self.items, object_fn, item)
    }
}

/// `bed_exclusion_applies_impl` on nester items (`Arrange.cpp:120-134`).
pub(crate) fn exclusion_applies(item: &Item, fixed_item: &Item) -> bool {
    if !fixed_item.is_bed_exclusion {
        return false;
    }
    let overlap = |first: [f64; 2], second: [f64; 2]| {
        let [a0, a1] = if first[0] > first[1] {
            [first[1], first[0]]
        } else {
            first
        };
        let [b0, b1] = if second[0] > second[1] {
            [second[1], second[0]]
        } else {
            second
        };
        a1 >= b0 - 1e-4 && b1 >= a0 - 1e-4
    };
    if item.has_z_range
        && fixed_item.has_z_range
        && !overlap(
            [item.z_min, item.z_max],
            [fixed_item.z_min, fixed_item.z_max],
        )
    {
        return false;
    }
    fixed_item.bed_exclusion_extruder_id < 0
        || item.bed_exclusion_extruder_ids.is_empty()
        || item
            .bed_exclusion_extruder_ids
            .contains(&fixed_item.bed_exclusion_extruder_id)
}

/// `minimizeCircle` (`nfpplacer.hpp:350-400`): the smallest circle round the contour by subplex
/// on the centre within a box-diagonal of the box centre (30 evaluations, 1e-3 relative).
pub(crate) fn bounding_circle(shape: &ExPolygon) -> (Point, f64) {
    let contour = shape.contour().points();
    if contour.is_empty() {
        return (Point::new(0, 0), 0.0);
    }
    let bb = shape_box(shape);
    let (center, reach) = (bb.center(), distance(bb.min, bb.max));
    let at = |x: &[f64]| {
        Point::new(
            (center.x() as f64 + reach * x[0]).round() as i64,
            (center.y() as f64 + reach * x[1]).round() as i64,
        )
    };
    let mut radius = |x: &[f64]| {
        let centre = at(x);
        contour
            .iter()
            .map(|&vertex| distance(vertex, centre))
            .fold(f64::MIN, f64::max)
    };
    let (optimum, score) = subplex_minimize(
        &mut radius,
        &[0.0, 0.0],
        (&[-1.0, -1.0], &[1.0, 1.0]),
        1e-3,
        30,
    );
    (at(&optimum), score)
}

/// `projection_onto(ExPolygons, Point)` (`ExPolygon.cpp:212-229`): the closest point on any
/// contour or hole.
pub(crate) fn projection_onto(shapes: &[ExPolygon], from: Point) -> Point {
    let mut projected = Point::new(0, 0);
    let mut min_dist = f64::MAX;
    for shape in shapes {
        for ring in std::iter::once(shape.contour()).chain(shape.holes()) {
            let p = project_onto_ring(ring.points(), from);
            let (dx, dy) = ((from.x() - p.x()) as f64, (from.y() - p.y()) as f64);
            if dx * dx + dy * dy < min_dist {
                projected = p;
                min_dist = dx * dx + dy * dy;
            }
        }
    }
    projected
}

/// `Point::projection_onto(MultiPoint)` (`Point.cpp:97-111`) over the closed ring's lines.
fn project_onto_ring(points: &[Point], from: Point) -> Point {
    let norm = |p: Point| ((p.x() - from.x()) as f64).hypot((p.y() - from.y()) as f64);
    let mut running = points[0];
    let mut running_min = norm(running);
    for index in 0..points.len() {
        let candidate = project_onto_line(points[index], points[(index + 1) % points.len()], from);
        if norm(candidate) < running_min {
            running = candidate;
            running_min = norm(running);
        }
    }
    running
}

/// `Point::projection_onto(Line)` (`Point.cpp:113-135`).
fn project_onto_line(a: Point, b: Point, from: Point) -> Point {
    if a == b {
        return a;
    }
    let (lx, ly) = ((b.x() - a.x()) as f64, (b.y() - a.y()) as f64);
    let theta =
        ((b.x() - from.x()) as f64 * lx + (b.y() - from.y()) as f64 * ly) / (lx * lx + ly * ly);
    if (0.0..=1.0).contains(&theta) {
        return Point::new(
            (theta * a.x() as f64 + (1.0 - theta) * b.x() as f64) as i64,
            (theta * a.y() as f64 + (1.0 - theta) * b.y() as f64) as i64,
        );
    }
    let squared = |p: Point| {
        let (dx, dy) = ((p.x() - from.x()) as f64, (p.y() - from.y()) as f64);
        dx * dx + dy * dy
    };
    if squared(a) < squared(b) { a } else { b }
}
