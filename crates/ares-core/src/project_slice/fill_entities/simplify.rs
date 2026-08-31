//! Fill extrusion simplification from `LayerRegion::simplify_*`
//! (`LayerRegion.cpp:1060-1125`).

use crate::{
    geometry::{CoordinateScale, Point, Polyline, douglas_peucker},
    project_slice::perimeters::classic::{
        gap_extrusion::GapFillEntity,
        materialize::{ExtrusionPath, Point3},
    },
};

use super::{FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn apply(output: &mut LayerFillEntities, resolution: f64, scale: CoordinateScale) {
    for collection in &mut output.collections {
        for entity in &mut collection.entities {
            match entity {
                FillExtrusionEntity::Path(path) => {
                    simplify_fill_path(
                        path,
                        resolution / scale.factor(),
                        collection.simplify_reversed,
                    );
                }
                FillExtrusionEntity::VariableWidth(entity) => {
                    simplify_gap_entity(entity, resolution / scale.factor());
                }
            }
        }
    }
}

fn simplify_fill_path(path: &mut FillExtrusionPath, tolerance: f64, reversed: bool) {
    let mut points = std::mem::replace(&mut path.polyline, Polyline::new(Vec::new())).into_points();
    if reversed {
        points.reverse();
    }
    let mut points = douglas_peucker(&points, tolerance);
    if reversed {
        points.reverse();
    }
    path.polyline = Polyline::new(points);
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
