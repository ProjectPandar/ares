//! Fill extrusion simplification from `LayerRegion::simplify_*`
//! (`LayerRegion.cpp:1060-1125`).

use crate::{
    ExtrusionRole,
    geometry::{CoordinateScale, Point, Polyline, douglas_peucker},
    project_slice::perimeters::classic::{
        gap_extrusion::GapFillEntity,
        materialize::{ExtrusionPath, Point3},
    },
};

use super::{FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn apply(
    output: &mut LayerFillEntities,
    resolution: f64,
    enable_arc_fitting: bool,
    scale: CoordinateScale,
) {
    for entity in output
        .collections
        .iter_mut()
        .flat_map(|collection| &mut collection.entities)
    {
        match entity {
            FillExtrusionEntity::Path(path) => {
                let tolerance = tolerance(path.role, resolution, enable_arc_fitting, scale);
                simplify_fill_path(path, tolerance);
            }
            FillExtrusionEntity::VariableWidth(entity) => {
                simplify_gap_entity(entity, resolution / scale.factor());
            }
        }
    }
}

fn tolerance(
    role: ExtrusionRole,
    resolution: f64,
    enable_arc_fitting: bool,
    scale: CoordinateScale,
) -> f64 {
    if enable_arc_fitting && role == ExtrusionRole::InternalInfill {
        0.04 / scale.factor()
    } else {
        resolution / scale.factor()
    }
}

fn simplify_fill_path(path: &mut FillExtrusionPath, tolerance: f64) {
    let points = std::mem::replace(&mut path.polyline, Polyline::new(Vec::new())).into_points();
    path.polyline = Polyline::new(douglas_peucker(&points, tolerance));
    path.fitting.clear();
}

fn simplify_gap_entity(entity: &mut GapFillEntity, tolerance: f64) {
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

#[cfg(test)]
mod tests;
