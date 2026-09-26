//! Debug dump helpers for `generate_layer` (env-gated diagnostic dumps).

use super::types::{FillExtrusionEntity, LayerFillEntities};
use crate::geometry::Point;

pub(super) fn plane_path_object_bounding_box(
    slices: &[Vec<crate::geometry::ExPolygon>],
) -> Option<crate::geometry::BoundingBox> {
    let mut points = slices
        .iter()
        .flatten()
        .flat_map(|expolygon| expolygon.contour().points());
    let first = *points.next()?;
    let (minimum, maximum) = points.fold((first, first), |(minimum, maximum), point| {
        (
            crate::geometry::Point::new(minimum.x().min(point.x()), minimum.y().min(point.y())),
            crate::geometry::Point::new(maximum.x().max(point.x()), maximum.y().max(point.y())),
        )
    });
    Some(crate::geometry::BoundingBox::new(minimum, maximum))
}

pub(super) fn dump_surfaces(path: &str, fill: &crate::project_slice::group_fills::SurfaceFill) {
    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    for expolygon in &fill.expolygons {
        let points = expolygon.contour().points();
        let head = points
            .iter()
            .take(3)
            .map(|point| format!(",{},{}", point.x(), point.y()))
            .collect::<String>();
        let _ = writeln!(
            file,
            "SURF pattern={:?} n={} bb={:?} {}",
            fill.params.pattern,
            points.len(),
            contour_bounds(points),
            head
        );
    }
}

pub(super) fn contour_bounds(points: &[Point]) -> [i64; 4] {
    let mut bounds = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    for point in points {
        bounds[0] = bounds[0].min(point.x());
        bounds[1] = bounds[1].min(point.y());
        bounds[2] = bounds[2].max(point.x());
        bounds[3] = bounds[3].max(point.y());
    }
    bounds
}

pub(super) fn dump_entities(path: &str, layer_index: usize, output: &LayerFillEntities) {
    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    for (index, collection) in output.collections.iter().enumerate() {
        let mut bounds = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
        let mut roles = Vec::new();
        for entity in &collection.entities {
            let FillExtrusionEntity::Path(path) = entity else {
                continue;
            };
            for point in path.polyline.points() {
                bounds[0] = bounds[0].min(point.x());
                bounds[1] = bounds[1].min(point.y());
                bounds[2] = bounds[2].max(point.x());
                bounds[3] = bounds[3].max(point.y());
            }
            roles.push(format!("{:?}", path.role));
        }
        let _ = writeln!(
            file,
            "ENT l={layer_index} c={index} bb={bounds:?} roles={roles:?}"
        );
    }
}
