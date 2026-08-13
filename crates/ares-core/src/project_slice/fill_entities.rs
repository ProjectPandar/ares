mod crosshatch;
mod types;

pub(in crate::project_slice) use types::{
    FillExtrusionCollection, FillExtrusionPath, LayerFillEntities,
};

use crate::{
    ProcessInfillPattern, SliceError,
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

pub(in crate::project_slice) fn generate_crosshatch_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<LayerFillEntities, SliceError> {
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
    let mut output = LayerFillEntities::default();

    for fill in grouped.surface_fills {
        match fill.params.pattern {
            SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
                crosshatch::append(&mut output, fill, z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(_) | SurfaceFillPattern::ConcentricInternal => {}
        }
    }
    Ok(output)
}
