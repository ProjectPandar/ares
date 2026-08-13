mod operation_order;
mod production;

use super::*;
use crate::{geometry::Point, project_slice::region_slices::RegionSurfaceKind};

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

fn flow(spacing: f32) -> Flow {
    Flow {
        width: spacing * 2.0,
        height: 0.2,
        spacing,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 1.0,
    }
}

fn snapshot(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
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

type SurfaceSnapshot = (
    RegionSurfaceKind,
    Vec<(i64, i64)>,
    Vec<Vec<(i64, i64)>>,
    u64,
    u16,
    u64,
    u16,
);

fn surface_snapshot(surfaces: &[RegionSurface]) -> Vec<SurfaceSnapshot> {
    surfaces
        .iter()
        .map(|surface| {
            let (kind, expolygon, thickness, thickness_layers, bridge_angle, extra_perimeters) =
                surface.as_parts();
            (
                kind,
                snapshot(std::slice::from_ref(expolygon.contour())).remove(0),
                snapshot(expolygon.holes()),
                thickness.to_bits(),
                thickness_layers,
                bridge_angle.to_bits(),
                extra_perimeters,
            )
        })
        .collect()
}
