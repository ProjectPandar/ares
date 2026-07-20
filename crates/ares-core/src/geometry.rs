mod chain_points;
mod clipper;
mod coord;
mod edge_grid;
mod expolygon;
mod polygon;
mod simplification;

pub(crate) use chain_points::chain_points;
pub(crate) use clipper::{
    ClipperError, FillRule, JoinType, difference_ex, difference_ex_with_safety_offset,
    intersection_ex, offset_expolygons, offset2_ex, union_ex, union_expolygons,
    variable_offset_inner_ex, xor_ex,
};
pub(crate) use coord::{Coord, CoordinateScale, Point};
pub(crate) use edge_grid::{EdgeGrid, GridEdge};
pub(crate) use expolygon::{ExPolygon, keep_largest_contour_only};
pub(crate) use polygon::Polygon;
pub(crate) use simplification::append_simplified_expolygon;

type BinaryExOperation = fn(&[ExPolygon], &[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>;
type ExPolygonsOffsetOperation =
    fn(&[ExPolygon], f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;
type VariableOffsetOperation =
    fn(&ExPolygon, &[Vec<f32>], f64) -> Result<Vec<ExPolygon>, ClipperError>;
type EdgeGridVisitor = fn(usize, usize, &[GridEdge]) -> bool;

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
const _: BinaryExOperation = difference_ex_with_safety_offset;
const _: BinaryExOperation = intersection_ex;
const _: BinaryExOperation = union_expolygons;
const _: BinaryExOperation = xor_ex;
const _: ExPolygonsOffsetOperation = offset_expolygons;
const _: VariableOffsetOperation = variable_offset_inner_ex;
const _: fn(ExPolygon, f64, &mut Vec<ExPolygon>) -> Result<(), ClipperError> =
    append_simplified_expolygon;
const _: fn(Point, Point, Point) -> f64 = simplification::distance_to_segment_squared;
const _: fn(&[Point], f64) -> Vec<Point> = simplification::douglas_peucker;
const _: fn(Vec<Point>, f64) -> Vec<Point> = simplification::simplify_closed_points;
const _: fn(&ExPolygon, Point, Point, Coord) -> Result<EdgeGrid, ClipperError> = EdgeGrid::new;
const _: fn(&EdgeGrid) -> (Point, Point) = EdgeGrid::bounds;
const _: fn(&EdgeGrid) -> Coord = EdgeGrid::resolution;
const _: fn(&EdgeGrid) -> (usize, usize) = EdgeGrid::dimensions;
const _: fn(&EdgeGrid, usize) -> &[Point] = EdgeGrid::contour;
const _: fn(&EdgeGrid, GridEdge) -> (Point, Point) = EdgeGrid::segment;
const _: fn(&EdgeGrid, Point, Point, EdgeGridVisitor) =
    EdgeGrid::visit_cells_intersecting_box::<EdgeGridVisitor>;

#[cfg(test)]
mod tests;
