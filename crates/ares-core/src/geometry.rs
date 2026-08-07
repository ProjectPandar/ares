mod bbox_clip;
mod bounding_box;
mod chain_points;
mod clipper;
mod coord;
mod edge_grid;
mod expolygon;
mod line;
pub(crate) mod medial_axis;
mod polygon;
mod polyline;
mod region_expansion;
mod simplification;

pub(crate) use bbox_clip::{
    clip_clipper_expolygons_with_subject_bbox, clip_clipper_polygons_with_subject_bbox,
};
pub(crate) use bounding_box::{BoundingBox, chain_expolygons, chain_expolygons_order};
pub(crate) use chain_points::chain_points;
#[cfg(test)]
pub(in crate::geometry) use clipper::opening_path_configurations_for_test;
#[cfg(test)]
pub(crate) use clipper::opening_paths_with_interstage;
pub(crate) use clipper::{
    ClipperError, FillRule, JoinType, SAFETY_OFFSET, diff_pl, difference_ex,
    difference_ex_polygons, difference_ex_polygons_with_safety_offset,
    difference_ex_with_safety_offset, difference_polygons_ex, difference_polygons_paths,
    intersection_ex, intersection_pl, intersection_polygons_ex, intersection_polygons_paths,
    intersection_polygons_paths_with_safety_offset, offset_expolygon, offset_expolygon_refs_paths,
    offset_expolygons, offset_expolygons_paths, offset_open_paths, offset_paths, offset_paths_tree,
    offset2_ex, offset2_ex_with_interstage, opening_ex, opening_paths, union_ex, union_expolygons,
    union_polygons_paths, variable_offset_inner_ex, xor_ex,
};
pub(crate) use coord::{Coord, CoordinateScale, Point};
pub(crate) use edge_grid::{EdgeGrid, GridEdge};
pub(crate) use expolygon::{ExPolygon, keep_largest_contour_only};
pub(crate) use line::{Line, ThickLine};
pub(crate) use medial_axis::medial_axis;
pub(crate) use polygon::Polygon;
pub(crate) use polyline::{Polyline, ThickPolyline, to_thick_polylines};
pub(crate) use region_expansion::{
    RegionExpansion, RegionExpansionEx, RegionExpansionParameters, WaveSeed, propagate_waves,
    propagate_waves_ex, propagate_waves_from_sources, propagate_waves_from_sources_with_steps,
};
#[cfg(test)]
pub(in crate::geometry) use region_expansion::{
    wavefront_counter_clockwise, wavefront_step_for_test,
};
pub(crate) use simplification::{append_simplified_expolygon, simplify_expolygon_polygons};

type BinaryExOperation = fn(&[ExPolygon], &[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>;
type PolygonClipOperation = fn(&[ExPolygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>;
type PolygonBinaryExOperation = fn(&[Polygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>;
type MixedPolygonExOperation = fn(&[Polygon], &[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>;
type ExPolygonsOffsetOperation =
    fn(&[ExPolygon], f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;
type VariableOffsetOperation =
    fn(&ExPolygon, &[Vec<f32>], f64) -> Result<Vec<ExPolygon>, ClipperError>;
type RegionExpansionOperation = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type RegionExpansionExOperation = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansionEx>, ClipperError>;
type RegionExpansionFromSourcesOperation = fn(
    &[ExPolygon],
    &[ExPolygon],
    &RegionExpansionParameters,
    CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type RegionExpansionFromSourcesWithStepsOperation =
    fn(
        &[ExPolygon],
        &[ExPolygon],
        f32,
        f32,
        usize,
        CoordinateScale,
    ) -> Result<Vec<RegionExpansion>, ClipperError>;
type EdgeGridVisitor = fn(usize, usize, &[GridEdge]) -> bool;

const _: usize = std::mem::size_of::<Coord>();
const _: fn(&Polygon) -> Option<BoundingBox> = BoundingBox::from_polygon;
const _: fn(&ExPolygon) -> Option<BoundingBox> = BoundingBox::from_expolygon;
const _: fn(BoundingBox) -> Point = BoundingBox::center;
const _: fn(&[ExPolygon]) -> Option<BoundingBox> = BoundingBox::from_expolygons;
const _: fn(&[Polygon]) -> Option<BoundingBox> = BoundingBox::from_polygons;
const _: fn(&mut BoundingBox, Coord) = BoundingBox::offset;
const _: fn(&[ExPolygon]) -> Vec<usize> = chain_expolygons_order;
const _: fn(Vec<ExPolygon>) -> Vec<ExPolygon> = chain_expolygons;
const _: fn(&[Polygon], BoundingBox) -> Vec<Polygon> = clip_clipper_polygons_with_subject_bbox;
const _: fn(&[ExPolygon], BoundingBox) -> Vec<Polygon> = clip_clipper_expolygons_with_subject_bbox;
const _: fn(&crate::Point2dList) -> CoordinateScale = CoordinateScale::from_printable_area;
const _: fn(CoordinateScale) -> f64 = CoordinateScale::factor;
const _: fn(CoordinateScale, f64) -> Option<Coord> = CoordinateScale::checked_scale;
const _: fn(CoordinateScale, Coord) -> f64 = CoordinateScale::unscale;
const _: fn(Coord, Coord) -> Point = Point::new;
const _: fn(Point) -> Coord = Point::x;
const _: fn(Point) -> Coord = Point::y;
const _: fn(Polygon) -> Vec<Point> = Polygon::into_points;
const _: fn(&Polygon) -> Polyline = Polygon::split_at_first_point;
const _: fn(&Polyline) -> bool = Polyline::is_valid;
const _: fn(Point, Point) -> ThickLine = ThickLine::new;
const _: fn(Point, Point, f64, f64) -> ThickLine = ThickLine::with_widths;
const _: fn(&mut ThickPolyline) = ThickPolyline::reverse;
const _: fn(&mut ThickPolyline) = ThickPolyline::clear;
const _: fn(&ThickPolyline) -> Vec<ThickLine> = ThickPolyline::thicklines;
const _: fn(&mut ThickPolyline, usize) = ThickPolyline::start_at_index;
const _: fn(Vec<Polyline>, f64) -> Vec<ThickPolyline> = to_thick_polylines;
type PolygonPolylineClip = fn(&[Polygon], &[Polygon]) -> Result<Vec<Polyline>, ClipperError>;
const _: PolygonPolylineClip = intersection_pl;
const _: PolygonPolylineClip = diff_pl;
const _: fn(&Polygon) -> f64 = Polygon::area;
const _: fn(Polygon, Vec<Polygon>) -> ExPolygon = ExPolygon::new;
const _: fn(&ExPolygon) -> &Polygon = ExPolygon::contour;
const _: fn(&ExPolygon) -> &[Polygon] = ExPolygon::holes;
const _: fn(ExPolygon) -> (Polygon, Vec<Polygon>) = ExPolygon::into_parts;
const _: fn(&mut Vec<ExPolygon>) = keep_largest_contour_only;
const _: BinaryExOperation = difference_ex;
const _: BinaryExOperation = difference_ex_with_safety_offset;
const _: PolygonClipOperation = difference_ex_polygons;
const _: PolygonClipOperation = difference_ex_polygons_with_safety_offset;
const _: PolygonBinaryExOperation = difference_polygons_ex;
const _: BinaryExOperation = intersection_ex;
const _: MixedPolygonExOperation = intersection_polygons_ex;
const _: BinaryExOperation = union_expolygons;
const _: BinaryExOperation = xor_ex;
const _: ExPolygonsOffsetOperation = offset_expolygons;
const _: VariableOffsetOperation = variable_offset_inner_ex;
const _: RegionExpansionOperation = propagate_waves;
const _: RegionExpansionExOperation = propagate_waves_ex;
const _: RegionExpansionFromSourcesOperation = propagate_waves_from_sources;
const _: RegionExpansionFromSourcesWithStepsOperation = propagate_waves_from_sources_with_steps;
const _: fn(f32, f32, usize, CoordinateScale) -> RegionExpansionParameters =
    RegionExpansionParameters::build;
const _: fn(ExPolygon, f64, &mut Vec<ExPolygon>) -> Result<(), ClipperError> =
    append_simplified_expolygon;
const _: fn(&ExPolygon, f64) -> Result<Vec<Polygon>, ClipperError> = simplify_expolygon_polygons;
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
