mod projected;

pub(in crate::project_slice) use projected::projected_sparse_density;
mod arms;
mod arms_tail;
mod honeycomb;
mod linear;
mod plane_path;

use crate::{
    SliceError,
    fill::multiline::Sweep,
    geometry::Polyline,
    project_slice::{
        group_fills::group_fills, prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
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
    grid_bounds: Option<crate::geometry::BoundingBox>,
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
    // Fill.cpp:1398,1442 uses the symmetric PrintObject box (Print.hpp:347).
    let object_reference = crate::geometry::Point::new(0, 0);
    let (object, print_object) = traversal.print_object_config(object_index);
    let object_bounding_box = crate::project_slice::fill_entities::print_object_bounds(
        &traversal.project.objects()[object.source_object_index],
        print_object.transform,
        traversal.scale,
    );
    let mut result = Vec::new();
    let spacing = &prepared.object_spacings[object_index];

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
            (grid_bounds, plan.layers[layer_index].id),
            spacing,
            &mut result,
        )?;
    }
    Ok(result)
}
