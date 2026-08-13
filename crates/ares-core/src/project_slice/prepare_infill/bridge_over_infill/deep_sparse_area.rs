#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the source-cited dependency remains unwired until the bridge transaction"
    )
)]

#[cfg(test)]
mod tests;

use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, closing_ex,
        difference_polygons_paths, union_expolygons,
    },
    project_slice::{
        layers::PlannedLayer,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const TARGET_FLOW_HEIGHT_FACTOR: f32 = 0.9;
const EPSILON: f64 = 1.0e-4;
const MITER_LIMIT: f64 = 3.0;

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct DeepSparseLayer<'a> {
    pub(in crate::project_slice) planned: &'a PlannedLayer,
    pub(in crate::project_slice) fill_surfaces: &'a [RegionSurface],
    pub(in crate::project_slice) sparse_infill_density_percent: f64,
}

pub(in crate::project_slice) fn gather_deep_sparse_infill_area(
    layers: &[DeepSparseLayer<'_>],
    candidate_layer_index: usize,
    target_flow_height: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    debug_assert!(candidate_layer_index > 0);
    debug_assert!(candidate_layer_index < layers.len());
    debug_assert!(target_flow_height.is_finite() && target_flow_height > 0.0);

    let bottom_z = layers[candidate_layer_index].planned.print_z
        - f64::from(target_flow_height * TARGET_FLOW_HEIGHT_FACTOR)
        - EPSILON;
    let mut sparse = Vec::new();
    let mut non_sparse = Vec::new();
    for layer_index in (0..candidate_layer_index).rev() {
        let layer = layers[layer_index];
        if layer.planned.print_z < bottom_z && layer_index < candidate_layer_index - 1 {
            break;
        }
        for surface in layer.fill_surfaces {
            let (kind, expolygon, ..) = surface.as_parts();
            if (kind == RegionSurfaceKind::Internal && layer.sparse_infill_density_percent < 100.0)
                || kind == RegionSurfaceKind::InternalVoid
            {
                sparse.push(expolygon.clone());
            } else {
                non_sparse.push(expolygon.clone());
            }
        }
    }

    let scaled_epsilon = scale
        .checked_scale(EPSILON)
        .expect("the fixed slicer epsilon fits every coordinate scale")
        as f32;
    let sparse = union_expolygons(&sparse, &[])?;
    let sparse = closing_ex(&sparse, scaled_epsilon, JoinType::Miter, MITER_LIMIT)?;
    let non_sparse = union_expolygons(&non_sparse, &[])?;
    let non_sparse = closing_ex(&non_sparse, scaled_epsilon, JoinType::Miter, MITER_LIMIT)?;
    difference_polygons_paths(
        &flatten_expolygons(&sparse),
        &flatten_expolygons(&non_sparse),
    )
}

fn flatten_expolygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|expolygon| {
            std::iter::once(expolygon.contour().clone()).chain(expolygon.holes().iter().cloned())
        })
        .collect()
}
