//! MultiMaterialSegmentation.cpp::segmentation_by_painting1979–2027.

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
    raw: &[Vec<ExPolygon>],
    scale: CoordinateScale,
) -> Result<Vec<Layer>, ClipperError> {
    let epsilon = scale.scaled_delta(1e-4);
    let area = scale.scaled_delta(f64::from(0.1_f32)).powi(2);
    let mut layers = Vec::with_capacity(raw.len());
    let mut bounds = Vec::with_capacity(raw.len());
    for slices in raw {
        let mut expanded = Vec::new();
        for slice in slices {
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
            BoundingBox::from_expolygons(slices),
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
        if index < raw.len() - 1 {
            bbox = merged(bbox, bounds[index + 1]);
        }
        let bbox = bbox.unwrap().inflated((20.0 * epsilon) as i64);
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
