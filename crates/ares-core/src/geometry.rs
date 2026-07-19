mod clipper;
mod coord;
mod expolygon;
mod polygon;
mod simplification;

pub(crate) use clipper::{
    ClipperError, FillRule, JoinType, difference_ex, intersection_ex, offset2_ex, union_ex,
};
pub(crate) use coord::{Coord, CoordinateScale, Point};
pub(crate) use expolygon::{ExPolygon, keep_largest_contour_only};
pub(crate) use polygon::Polygon;
pub(crate) use simplification::append_simplified_expolygon;

type BinaryExOperation = fn(&[ExPolygon], &[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>;

const _: usize = std::mem::size_of::<Coord>();
const _: fn(&crate::Point2dList) -> CoordinateScale = CoordinateScale::from_printable_area;
const _: fn(CoordinateScale) -> f64 = CoordinateScale::factor;
const _: fn(CoordinateScale, f64) -> Option<Coord> = CoordinateScale::checked_scale;
const _: fn(CoordinateScale, Coord) -> f64 = CoordinateScale::unscale;
const _: fn(Coord, Coord) -> Point = Point::new;
const _: fn(Point) -> Coord = Point::x;
const _: fn(Point) -> Coord = Point::y;
const _: fn(Polygon) -> Vec<Point> = Polygon::into_points;
const _: fn(&Polygon) -> f64 = Polygon::area;
const _: fn(Polygon, Vec<Polygon>) -> ExPolygon = ExPolygon::new;
const _: fn(&ExPolygon) -> &Polygon = ExPolygon::contour;
const _: fn(&ExPolygon) -> &[Polygon] = ExPolygon::holes;
const _: fn(ExPolygon) -> (Polygon, Vec<Polygon>) = ExPolygon::into_parts;
const _: fn(&mut Vec<ExPolygon>) = keep_largest_contour_only;
const _: BinaryExOperation = difference_ex;
const _: BinaryExOperation = intersection_ex;
const _: fn(ExPolygon, f64, &mut Vec<ExPolygon>) -> Result<(), ClipperError> =
    append_simplified_expolygon;
const _: fn(Point, Point, Point) -> f64 = simplification::distance_to_segment_squared;
const _: fn(&[Point], f64) -> Vec<Point> = simplification::douglas_peucker;
const _: fn(Vec<Point>, f64) -> Vec<Point> = simplification::simplify_closed_points;

#[cfg(test)]
mod tests;
