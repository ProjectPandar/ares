//! `PrintObject::detect_overhangs_for_lift` (`PrintObject.cpp:818–848`).

use super::Layer;
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, difference_ex,
    offset_expolygons, offset2_ex,
};
use crate::project_slice::layers::PlannedPrintObject;

pub(super) fn layers(
    slices: &[Vec<ExPolygon>],
    plan: &PlannedPrintObject,
    line_width: f64,
    scale: CoordinateScale,
) -> Result<Vec<Layer>, ClipperError> {
    let overlap = (line_width * f64::from(0.3_f32)) as f32;
    let overlap = (f64::from(overlap) / scale.factor()) as f32;
    let opening = (f64::from(0.1_f32) * (line_width / scale.factor())) as f32;
    let raft =
        plan.slicing_parameters.base_raft_layers + plan.slicing_parameters.interface_raft_layers;
    let mut layers = Vec::with_capacity(plan.layers.len());
    for (index, layer) in plan.layers.iter().enumerate() {
        let polygons = if index > raft {
            let lower = offset_expolygons(&slices[index - 1], overlap, JoinType::Miter, 3.0)?;
            let overhangs = difference_ex(&slices[index], &lower)?;
            offset2_ex(&overhangs, -opening, opening, JoinType::Miter, 3.0)?
                .into_iter()
                .flat_map(|polygon| {
                    let (contour, holes) = polygon.into_parts();
                    std::iter::once(contour).chain(holes)
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        layers.push(Layer {
            print_z: layer.print_z,
            bounds: BoundingBox::from_polygons(&polygons),
            polygons,
        });
    }
    Ok(layers)
}
