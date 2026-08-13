mod errors;
mod membership;
mod morphology;
mod operation_order;

use super::{
    CurrentLayerBridgeContext, CurrentLayerBridgeRegion, clip_lower_lines_using,
    intersect_expansion_with_deep_using, prepare_current_layer_bridge_context,
};
use crate::{
    ProcessInfillPattern,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon, Polyline},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn prepare(
    deep: &[Polygon],
    regions: &[CurrentLayerBridgeRegion<'_>],
    lines: &[Polyline],
    spacing: i64,
    scale: CoordinateScale,
) -> CurrentLayerBridgeContext {
    prepare_current_layer_bridge_context(deep, regions, lines, spacing, scale).unwrap()
}

fn region<'a>(
    surfaces: &'a [RegionSurface],
    fill: &'a [ExPolygon],
    pattern: ProcessInfillPattern,
) -> CurrentLayerBridgeRegion<'a> {
    CurrentLayerBridgeRegion {
        fill_surfaces: surfaces,
        fill_expolygons: fill,
        sparse_infill_pattern: pattern,
    }
}

fn surface(kind: RegionSurfaceKind, polygon: Polygon) -> RegionSurface {
    RegionSurface::new(kind, ExPolygon::new(polygon, Vec::new()))
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn expolygon(contour: Polygon, holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(contour, holes)
}

fn line(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn outside_range() -> Polygon {
    rectangle(HI_RANGE, 0, HI_RANGE + 10, 10)
}

fn snapshot_polygons(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

fn snapshot_polylines(polylines: &[Polyline]) -> Vec<Vec<(i64, i64)>> {
    polylines
        .iter()
        .map(|polyline| {
            polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

fn bounds(polygons: &[Polygon]) -> (i64, i64, i64, i64) {
    let mut points = polygons.iter().flat_map(Polygon::points);
    let first = points.next().unwrap();
    points.fold(
        (first.x(), first.y(), first.x(), first.y()),
        |(min_x, min_y, max_x, max_y), point| {
            (
                min_x.min(point.x()),
                min_y.min(point.y()),
                max_x.max(point.x()),
                max_y.max(point.y()),
            )
        },
    )
}

fn total_area(polygons: &[Polygon]) -> f64 {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .zip(polygon.points().iter().cycle().skip(1))
                .map(|(a, b)| (a.x() as f64 * b.y() as f64) - (b.x() as f64 * a.y() as f64))
                .sum::<f64>()
                .abs()
                * 0.5
        })
        .sum()
}
