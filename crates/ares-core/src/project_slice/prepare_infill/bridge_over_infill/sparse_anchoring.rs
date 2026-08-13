mod grouping;
#[cfg(test)]
mod tests;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, RegionOptions,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface},
    geometry::{ClipperError, CoordinateScale, Polyline},
    project_slice::{
        layers::PlannedLayer,
        perimeters::flow::resolve_nominal_sparse_infill_flow,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct SparseAnchoringLayer<'a> {
    pub(in crate::project_slice) planned: &'a PlannedLayer,
    pub(in crate::project_slice) fill_surfaces: &'a [RegionSurface],
    pub(in crate::project_slice) region_options: &'a RegionOptions,
    pub(in crate::project_slice) object_options: &'a ObjectOptions,
    pub(in crate::project_slice) nozzle_diameters: &'a OrcaFloats,
    pub(in crate::project_slice) scale: CoordinateScale,
}

pub(in crate::project_slice) fn generate_sparse_infill_polylines_for_anchoring(
    layer: SparseAnchoringLayer<'_>,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(layer.region_options.sparse_infill_density.0 > 0.0);
    debug_assert!(layer.region_options.top_surface_density.0 > 0.0);
    debug_assert!(
        layer
            .region_options
            .sparse_infill_rotate_template
            .0
            .is_empty()
    );
    debug_assert!(
        layer
            .region_options
            .solid_infill_rotate_template
            .0
            .is_empty()
    );
    debug_assert!(!layer.region_options.align_infill_direction_to_model.0);
    debug_assert_eq!(layer.region_options.fill_multiline.0, 1);

    let flow = resolve_nominal_sparse_infill_flow(
        layer.region_options,
        layer.object_options,
        layer.nozzle_diameters,
    )
    .expect("bridge transaction validates the nominal frInfill Flow before O46");
    let spacing = f64::from(flow.spacing);
    let density = projected_sparse_density(layer.region_options);
    let angle = layer.region_options.infill_direction.0.to_radians() as f32;
    let (anchor_length, anchor_length_max) =
        projected_anchor_lengths(layer.region_options, spacing);
    debug_assert!(anchor_length_max >= 0.05);

    let groups = grouping::group_and_prioritize(layer.fill_surfaces, layer.region_options)?;
    let params = CrossHatchFillParams {
        z: layer.planned.print_z,
        spacing,
        overlap: 0.0,
        angle,
        density,
        multiline: 1,
        anchor_length,
        anchor_length_max,
        dont_sort: false,
    };
    let mut result = Vec::new();
    for group in groups {
        if group.representative_kind != RegionSurfaceKind::Internal {
            continue;
        }
        match group.pattern {
            grouping::Pattern::CrossHatch => {
                for expolygon in group.expolygons {
                    result.extend(fill_surface(&expolygon, params, layer.scale)?);
                }
            }
            grouping::Pattern::Monotonic | grouping::Pattern::MonotonicLine => {
                unreachable!("trusted Internal sparse anchoring group is CrossHatch")
            }
        }
    }
    Ok(result)
}

pub(super) fn projected_sparse_density(options: &RegionOptions) -> f32 {
    let density_percent = options.sparse_infill_density.0 as f32;
    (0.01_f64 * f64::from(density_percent)) as f32
}

pub(super) fn projected_anchor_lengths(options: &RegionOptions, spacing: f64) -> (f32, f32) {
    let anchor_length = projected_length(options.infill_anchor, spacing);
    let anchor_length_max = projected_length(options.infill_anchor_max, spacing);
    (anchor_length.min(anchor_length_max), anchor_length_max)
}

fn projected_length(value: FloatOrPercent, spacing: f64) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => (f64::from(value.0 as f32) * 0.01_f64 * spacing) as f32,
    }
}
