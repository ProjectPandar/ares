//! MultiMaterialSegmentation.cpp::segmentation_by_painting1979–2027.

use super::super::PostRegionPrintObject;
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, EdgeGrid, ExPolygon, JoinType,
    append_simplified_expolygon, mutable_polygon::remove_duplicates, offset_expolygon,
    offset_expolygons, union_expolygons,
};

pub(super) struct Layer {
    pub(super) input: Vec<ExPolygon>,
    pub(super) grid: Option<EdgeGrid>,
}

fn merged(a: Option<BoundingBox>, b: Option<BoundingBox>) -> Option<BoundingBox> {
    match (a, b) {
        (Some(mut a), Some(b)) => {
            a.merge_point(b.min());
            a.merge_point(b.max());
            Some(a)
        }
        (a, None) => a,
        (None, b) => b,
    }
}

pub(super) fn prepare(
    object: &PostRegionPrintObject,
    scale: CoordinateScale,
) -> Result<Vec<Layer>, ClipperError> {
    let epsilon = scale.scaled_delta(1e-4);
    let area = scale.scaled_delta(f64::from(0.1_f32)).powi(2);
    let count = object.plan.layers.len();
    let mut layers = Vec::with_capacity(count);
    let mut bounds = Vec::with_capacity(count);
    for index in 0..count {
        let mut expanded = Vec::new();
        let mut original_bounds = None;
        for slice in object.regions.iter().flat_map(|region| {
            region.layers[index]
                .surfaces
                .iter()
                .map(|surface| surface.as_parts().1)
        }) {
            original_bounds = merged(original_bounds, BoundingBox::from_expolygon(slice));
            expanded.extend(offset_expolygon(
                slice,
                (10.0 * epsilon) as f32,
                JoinType::Miter,
                3.0,
            )?);
        }
        let mut polygons = union_expolygons(&expanded, &[])?;
        polygons.retain_mut(|p| {
            if p.area().abs() < area {
                return false;
            }
            p.holes_mut().retain(|hole| hole.area().abs() >= area);
            true
        });
        let shrunk =
            offset_expolygons(&polygons, -10.0_f32 * epsilon as f32, JoinType::Miter, 3.0)?;
        let mut simplified = Vec::new();
        for polygon in shrunk {
            append_simplified_expolygon(polygon, 5.0 * epsilon, &mut simplified)?;
        }
        let input = remove_duplicates(
            simplified,
            scale.scaled_delta(0.01) as i64,
            std::f64::consts::PI / 6.0,
        );
        bounds.push(merged(
            original_bounds,
            BoundingBox::from_expolygons(&input),
        ));
        layers.push(Layer { input, grid: None });
    }
    for (index, layer) in layers.iter_mut().enumerate() {
        if layer.input.is_empty() {
            continue;
        }
        let mut bbox = bounds[index];
        // The source deliberately does not merge layer 0 into layer 1 here.
        if index > 1 {
            bbox = merged(bbox, bounds[index - 1]);
        }
        if index < count - 1 {
            bbox = merged(bbox, bounds[index + 1]);
        }
        // BoundingBox::offset(double) constructs the rounding Point(double, double).
        let bbox = bbox.unwrap().inflated((20.0 * epsilon).round() as i64);
        layer.grid = Some(EdgeGrid::new_from_contours(
            layer
                .input
                .iter()
                .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
                .map(|p| p.points()),
            bbox.min(),
            bbox.max(),
            scale.scaled_delta(10.0) as i64,
        )?);
    }
    Ok(layers)
}
