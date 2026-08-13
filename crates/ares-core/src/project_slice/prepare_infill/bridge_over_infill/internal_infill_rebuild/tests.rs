mod operation_order;
mod production;

use super::*;
use crate::geometry::Point;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ])
}

fn expolygon(contour: Polygon, holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(contour, holes)
}

fn surface(kind: RegionSurfaceKind, expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::new(kind, expolygon)
}

type PointsSnapshot = Vec<(i64, i64)>;
type ExPolygonSnapshot = (PointsSnapshot, Vec<PointsSnapshot>);
type SurfaceSnapshot = (RegionSurfaceKind, PointsSnapshot, u64, u16, u64, u16);

fn snapshot_ex(expolygons: &[ExPolygon]) -> Vec<ExPolygonSnapshot> {
    expolygons
        .iter()
        .map(|expolygon| {
            (
                expolygon
                    .contour()
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect(),
                expolygon
                    .holes()
                    .iter()
                    .map(|hole| {
                        hole.points()
                            .iter()
                            .map(|point| (point.x(), point.y()))
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect()
}

fn surface_snapshot(surfaces: &[RegionSurface]) -> Vec<SurfaceSnapshot> {
    surfaces
        .iter()
        .map(|surface| {
            let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            (
                kind,
                expolygon
                    .contour()
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect(),
                thickness.to_bits(),
                layers,
                angle.to_bits(),
                extra,
            )
        })
        .collect()
}
