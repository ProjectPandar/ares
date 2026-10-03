//! `SupportMaterialInternal::collect_bridging_perimeter_areas`.

use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Point, Polygon, offset_open_paths, offset_paths,
};
use crate::project_slice::perimeters::classic::{
    entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
    materialize::{ExtrusionPath, ExtrusionRole},
};

pub(super) fn collect_bridging_perimeter_areas(
    perimeters: &[ExtrusionEntityCollection],
    expansion_scaled: f32,
    scale: CoordinateScale,
    output: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    for entity in perimeters
        .iter()
        .flat_map(|collection| &collection.entities)
    {
        if let ExtrusionEntity::Loop(loop_) = entity {
            for path in &loop_.extrusion_loop.paths {
                append_path_area(path, expansion_scaled, scale, output)?;
            }
        }
    }
    Ok(())
}

fn append_path_area(
    path: &ExtrusionPath,
    expansion_scaled: f32,
    scale: CoordinateScale,
    output: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    let points = &path.polyline.points;
    if path.role != ExtrusionRole::OverhangPerimeter || points.is_empty() {
        return Ok(());
    }
    let expansion = 0.5_f32 * (f64::from(path.width) / scale.factor()) as f32 + expansion_scaled;
    let closed = points[0] == points[points.len() - 1];
    if closed {
        if points.len() >= 3 {
            let mut polygon = Polygon::new(
                points[..points.len() - 1]
                    .iter()
                    .map(|point| Point::new(point.x, point.y))
                    .collect(),
            );
            if polygon.area() < 0.0 {
                polygon.reverse();
            }
            output.extend(offset_paths(
                std::slice::from_ref(&polygon),
                expansion,
                JoinType::Square,
                0.0,
            )?);
            let mut holes = offset_paths(&[polygon], -expansion, JoinType::Square, 0.0)?;
            for hole in &mut holes {
                hole.reverse();
            }
            output.append(&mut holes);
        }
    } else if points.len() >= 2 {
        let polygon = Polygon::new(
            points
                .iter()
                .map(|point| Point::new(point.x, point.y))
                .collect(),
        );
        output.extend(offset_open_paths(
            &[polygon],
            expansion,
            JoinType::Square,
            0.0,
        )?);
    }
    Ok(())
}
