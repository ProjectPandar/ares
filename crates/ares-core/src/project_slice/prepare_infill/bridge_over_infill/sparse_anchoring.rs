use crate::{
    FloatOrPercent, ProcessInfillPattern, RegionOptions, SliceError,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface},
    geometry::Polyline,
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
        region_slices::RegionSurfaceKind,
    },
};

pub(in crate::project_slice) fn generate_sparse_infill_polylines_for_anchoring(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<Vec<Polyline>, SliceError> {
    let grouped = group_fills(prepared, object_index, layer_index)?;
    let traversal = &prepared.predecessor.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let z = plan.layers[layer_index].print_z;
    let mut result = Vec::new();

    for fill in grouped.surface_fills {
        if fill.representative.kind != RegionSurfaceKind::Internal {
            continue;
        }
        match fill.params.pattern {
            SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
                let params = CrossHatchFillParams {
                    z,
                    spacing: fill.params.spacing,
                    overlap: 0.0,
                    angle: fill.params.angle,
                    density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                    multiline: fill.params.multiline,
                    anchor_length: fill.params.anchor_length,
                    anchor_length_max: fill.params.anchor_length_max,
                    dont_sort: false,
                };
                for expolygon in fill.expolygons {
                    result.extend(
                        fill_surface(&expolygon, params, traversal.scale)
                            .map_err(super::transaction::geometry_error)?,
                    );
                }
            }
            SurfaceFillPattern::Configured(
                ProcessInfillPattern::Rectilinear
                | ProcessInfillPattern::Monotonic
                | ProcessInfillPattern::MonotonicLine
                | ProcessInfillPattern::AlignedRectilinear
                | ProcessInfillPattern::ZigZag
                | ProcessInfillPattern::CrossZag
                | ProcessInfillPattern::LockedZag
                | ProcessInfillPattern::Line
                | ProcessInfillPattern::Grid
                | ProcessInfillPattern::Triangles
                | ProcessInfillPattern::TriHexagon
                | ProcessInfillPattern::Cubic
                | ProcessInfillPattern::AdaptiveCubic
                | ProcessInfillPattern::QuarterCubic
                | ProcessInfillPattern::SupportCubic
                | ProcessInfillPattern::Lightning
                | ProcessInfillPattern::Honeycomb
                | ProcessInfillPattern::ThreeDHoneycomb
                | ProcessInfillPattern::LateralHoneycomb
                | ProcessInfillPattern::LateralLattice
                | ProcessInfillPattern::TpmsD
                | ProcessInfillPattern::TpmsFk
                | ProcessInfillPattern::Gyroid
                | ProcessInfillPattern::Concentric
                | ProcessInfillPattern::HilbertCurve
                | ProcessInfillPattern::ArchimedeanChords
                | ProcessInfillPattern::OctagramSpiral,
            )
            | SurfaceFillPattern::ConcentricInternal => {
                unreachable!("bridge transaction admits only CrossHatch sparse anchoring")
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
