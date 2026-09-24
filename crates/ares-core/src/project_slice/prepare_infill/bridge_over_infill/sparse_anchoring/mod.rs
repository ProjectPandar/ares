mod projected;

pub(in crate::project_slice) use projected::{
    object_bounding_box, projected_anchor_lengths, projected_sparse_density,
};
mod arms;
mod arms_tail;

use crate::{
    ProcessInfillPattern, SliceError,
    fill::{
        cross_hatch::{CrossHatchFillParams, fill_surface as fill_cross_hatch},
        gyroid::{GyroidFillParams, fill_surface as fill_gyroid},
        multiline::{MultilineFillParams, Sweep, fill_surface as fill_multiline_surface},
        rectilinear::{MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface},
        three_d_honeycomb::{Params as Honeycomb3dParams, fill_surface as fill_honeycomb_3d},
    },
    geometry::{Point, Polyline},
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
        region_slices::RegionSurfaceKind,
    },
};

const GRID_SWEEPS: [Sweep; 2] = [
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_2,
        shift: 0.0,
    },
];
const TRIANGLE_SWEEPS: [Sweep; 3] = [
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
    Sweep {
        angle: 2.0 * std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
];

pub(in crate::project_slice) fn generate_sparse_infill_polylines_for_anchoring(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
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
    let (post_regions, object_slices) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let z = plan.layers[layer_index].print_z;
    let object_reference = crate::project_slice::fill_entities::object_center(object_slices);
    // Empirically the consistent-pattern grid anchor + plane-path `out_shift`
    // match the slice-contour extent, not the raw mesh `m_size` (see
    // `fill_entities/generate.rs`). Keep the contour-derived bbox.
    let object_bounding_box = object_bounding_box(object_slices);
    let mut result = Vec::new();

    let multiline_params =
        |params: crate::project_slice::group_fills::SurfaceFillParams| MultilineFillParams {
            spacing: params.spacing,
            overlap: 0.0,
            angle: params.angle + std::f32::consts::FRAC_PI_2,
            reference: object_reference,
            density: (0.01_f64 * f64::from(params.density)) as f32,
            multiline: params.multiline,
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            dont_sort: false,
        };

    for fill in grouped.surface_fills {
        if fill.representative.kind != RegionSurfaceKind::Internal {
            continue;
        }
        arms::dispatch(
            fill,
            traversal,
            object_index,
            layer_index,
            z,
            object_reference,
            object_bounding_box,
            lightning_generator,
            &mut result,
        )?;
    }
    Ok(result)
}
