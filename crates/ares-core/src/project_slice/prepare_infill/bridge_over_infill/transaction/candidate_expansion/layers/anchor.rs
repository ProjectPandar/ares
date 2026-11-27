use crate::{
    SliceError,
    geometry::Polyline,
    project_slice::{
        perimeters::{flow::resolve_nominal_sparse_infill_flow, types::RegionLayerIndex},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::super::super::super::sparse_anchoring::{
    AnchoringContext, generate_sparse_infill_polylines_for_anchoring,
};
use super::super::super::anchor_projection;
use super::super::ObjectView;

pub(super) fn prepare_anchor_layer(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    lower_layer: usize,
    view: ObjectView<'_>,
    context: AnchoringContext<'_>,
) -> Result<Vec<Polyline>, SliceError> {
    let ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
        object_options,
        ..
    } = view;
    let (_, inputs) = prelude.as_parts();
    let region_count = prelude.as_parts().0.as_parts().0.as_parts().2.len();
    for region_index in 0..region_count {
        let slot = prelude.region_layer_record_index(RegionLayerIndex {
            region_index,
            layer_index: lower_layer,
        });
        let input = match (&inputs[slot], &horizontal.records[slot]) {
            (Some(input), Some(_)) => input,
            (None, None) => continue,
            _ => unreachable!("bridge anchor records remain aligned"),
        };
        let region = prelude.region_options(input);
        if region.sparse_infill_density.0 == 0.0 {
            continue;
        }
        let nominal_flow = resolve_nominal_sparse_infill_flow(region, object_options, nozzles)?;
        anchor_projection::validate(region, nominal_flow.spacing, scale)?;
    }
    generate_sparse_infill_polylines_for_anchoring(predecessor, object_index, lower_layer, context)
}
