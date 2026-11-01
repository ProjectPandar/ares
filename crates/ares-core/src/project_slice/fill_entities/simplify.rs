//! Fill extrusion simplification from `LayerRegion::simplify_*`
//! (`LayerRegion.cpp:1060-1125`).

use crate::{
    geometry::{CoordinateScale, Point, douglas_peucker},
    project_slice::perimeters::classic::{
        gap_extrusion::GapFillEntity,
        materialize::{ExtrusionPath, Point3},
    },
};

use super::{FillExtrusionEntity, FillExtrusionPath, LayerFillEntities, simplify_before_ordering};

pub(super) fn apply(
    output: &mut LayerFillEntities,
    resolution: f64,
    scale: CoordinateScale,
    arc_fitting: crate::ArcFittingMode,
    spiral_mode: bool,
) {
    for collection in &mut output.collections {
        for entity in &mut collection.entities {
            match entity {
                FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path)
                    if simplify_before_ordering(
                        arc_fitting,
                        spiral_mode,
                        path.role == crate::ExtrusionRole::InternalInfill,
                    ) =>
                {
                    simplify_fill_path(path, resolution / scale.factor());
                }
                FillExtrusionEntity::Path(_) | FillExtrusionEntity::Oriented(_) => {}
                FillExtrusionEntity::VariableWidth(entity) => {
                    simplify_gap_entity(
                        entity,
                        resolution / scale.factor(),
                        arc_fitting,
                        spiral_mode,
                    );
                }
            }
        }
    }
}

fn simplify_fill_path(path: &mut FillExtrusionPath, tolerance: f64) {
    path.polyline.points = crate::geometry::simplification::douglas_peucker_projected(
        &path.polyline.points,
        tolerance,
        |point| Point::new(point.x, point.y),
    );
    path.polyline.fitting.clear();
}

fn simplify_gap_entity(
    entity: &mut GapFillEntity,
    tolerance: f64,
    arc_fitting: crate::ArcFittingMode,
    spiral_mode: bool,
) {
    if !simplify_before_ordering(arc_fitting, spiral_mode, false) {
        return;
    }
    match entity {
        GapFillEntity::Path(path) => simplify_extrusion_path(path, tolerance),
        GapFillEntity::Loop(paths) => {
            for path in paths {
                simplify_extrusion_path(path, tolerance);
            }
        }
    }
}

fn simplify_extrusion_path(path: &mut ExtrusionPath, tolerance: f64) {
    let points = path
        .polyline
        .points
        .iter()
        .map(|point| Point::new(point.x, point.y))
        .collect::<Vec<_>>();
    let z = path.polyline.points.first().map_or(0, |point| point.z);
    path.polyline.points = douglas_peucker(&points, tolerance)
        .into_iter()
        .map(|point| Point3 {
            x: point.x(),
            y: point.y(),
            z,
        })
        .collect();
    path.polyline.fitting.clear();
}
