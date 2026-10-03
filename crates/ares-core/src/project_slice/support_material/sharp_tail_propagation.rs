//! Serial sharp-tail propagation in `SupportMaterial.cpp::top_contact_layers`.

use super::{
    layer_input::{LayerAnnotations, LayerInput},
    overhangs::{LENGTH_THRESH_WELL_SUPPORTED, SharpTails, overlaps},
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, difference_ex,
    difference_ex_polygons, intersection_ex, offset_expolygons,
};

pub(super) struct PropagationInput<'a> {
    pub(super) layers: &'a [LayerInput<'a>],
    pub(super) annotations: &'a [LayerAnnotations<'a>],
    pub(super) start_layer: usize,
    pub(super) extrusion_width_scaled: f64,
    pub(super) scale: CoordinateScale,
}

pub(super) fn propagate_sharp_tails(
    input: &PropagationInput<'_>,
    sharp_tails: &mut [SharpTails],
    overhangs: &mut [Vec<ExPolygon>],
) -> Result<(), ClipperError> {
    for layer_index in input.start_layer.max(1)..input.layers.len() {
        let (lower_layers, current_layers) = sharp_tails.split_at_mut(layer_index);
        let lower = &lower_layers[layer_index - 1];
        let current = &mut current_layers[0];
        let lower_size = extent_size(&lower.polygons);
        for polygon in input.layers[layer_index].slices {
            let polygon_slice = std::slice::from_ref(polygon);
            let supported = intersection_ex(polygon_slice, &lower.polygons)?;
            if supported.is_empty() {
                continue;
            }
            let supported_size = extent_size(&supported);
            if supported_size.x() as f64 > LENGTH_THRESH_WELL_SUPPORTED
                && supported_size.y() as f64 > LENGTH_THRESH_WELL_SUPPORTED
            {
                continue;
            }
            let mut accumulated_height = input.layers[layer_index].height as f32;
            if let Some(height) = first_overlapping_height(lower, polygon)? {
                accumulated_height += height;
            }
            if accumulated_height > 16.0 {
                continue;
            }
            let new_overhang = difference_ex(polygon_slice, &lower.polygons)?;
            let new_size = extent_size(&new_overhang);
            // Point(scale_(5), scale_(5)) rounds the native double coordinates.
            let growth_threshold = (5.0 / input.scale.factor()).round() as i64;
            if ((new_size.x() - lower_size.x()) > growth_threshold
                && (new_size.y() - lower_size.y()) > growth_threshold)
                || !offset_expolygons(
                    &new_overhang,
                    (-5.0 * input.extrusion_width_scaled) as f32,
                    JoinType::Miter,
                    3.0,
                )?
                .is_empty()
            {
                continue;
            }
            let mut overhang = difference_ex(polygon_slice, input.layers[layer_index - 1].slices)?;
            current.polygons.push(polygon.clone());
            current.heights.push(accumulated_height);
            let blockers = input.annotations[layer_index].blockers;
            if !blockers.is_empty() {
                overhang = difference_ex_polygons(&overhang, blockers)?;
            }
            overhangs[layer_index].extend(overhang);
        }
    }
    Ok(())
}

fn first_overlapping_height(
    tails: &SharpTails,
    polygon: &ExPolygon,
) -> Result<Option<f32>, ClipperError> {
    for (index, &height) in tails.heights.iter().enumerate() {
        let lower_polygon = &tails.polygons[index];
        if overlaps(
            std::slice::from_ref(lower_polygon),
            std::slice::from_ref(polygon),
        )? {
            return Ok(Some(height));
        }
    }
    Ok(None)
}

pub(super) fn extent_size(polygons: &[ExPolygon]) -> Point {
    // Native get_extents(empty) has a zero-initialized BoundingBox size.
    BoundingBox::from_expolygons(polygons).map_or(Point::new(0, 0), |bounds| {
        Point::new(
            bounds.max().x() - bounds.min().x(),
            bounds.max().y() - bounds.min().y(),
        )
    })
}
