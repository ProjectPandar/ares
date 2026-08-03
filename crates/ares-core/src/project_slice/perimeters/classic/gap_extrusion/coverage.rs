use crate::geometry::{ClipperError, CoordinateScale, JoinType, Point, Polygon, offset_open_paths};

use super::{ExtrusionPath, GapFillCollection, GapFillEntity};

pub(in crate::project_slice) fn covered_polygons(
    collection: &GapFillCollection,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::new();
    for entity in &collection.entities {
        match entity {
            GapFillEntity::Path(path) => output.append(&mut covered_path(path, scale)?),
            GapFillEntity::Loop(paths) => {
                for path in paths {
                    output.append(&mut covered_path(path, scale)?);
                }
            }
        }
    }
    Ok(output)
}

fn covered_path(
    path: &ExtrusionPath,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let points = path
        .polyline
        .points
        .iter()
        .map(|point| Point::new(point.x, point.y))
        .collect();
    let delta = (f64::from(path.width / 2.0) / scale.factor()) as f32 + 10.0;
    offset_open_paths(&[Polygon::new(points)], delta, JoinType::Square, 0.0)
}
