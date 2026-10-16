use super::super::prepare_infill::external_surfaces::PreparedPostExternalSurfaces;
use super::generate::{LayerGenerationOptions, generate_layer_with_active};
use super::types::LayerFillEntities;
use crate::{ProcessInfillPattern, SliceError};

pub(in crate::project_slice) fn generate_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
    stars_bounds: Option<crate::geometry::BoundingBox>,
) -> Result<LayerFillEntities, SliceError> {
    let traversal = &prepared.predecessor.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
    let active_region_ids = super::adaptive::active_region_ids(
        prelude,
        &prepared.predecessor.objects[object_index].records,
        plan.layers.len(),
    );
    let resolved = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == prelude.identity().0)
        .expect("fill generation retains its resolved object");
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    generate_layer_with_active(
        prepared,
        object_index,
        layer_index,
        LayerGenerationOptions {
            lightning_generator,
            stars_bounds,
            adaptive_spacing: super::adaptive::line_spacing::object_configured(
                &resolved.all_regions,
                nozzles,
                ProcessInfillPattern::AdaptiveCubic,
                &active_region_ids,
            ),
            support_spacing: super::adaptive::line_spacing::object_configured(
                &resolved.all_regions,
                nozzles,
                ProcessInfillPattern::SupportCubic,
                &active_region_ids,
            ),
        },
    )
}
