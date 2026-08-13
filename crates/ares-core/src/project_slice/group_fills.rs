mod coalesce;
mod params;
mod priority;
mod types;

pub(in crate::project_slice) use types::{
    BaseGroupedFills, LockDensityParam, LockFlowParam, LockRegionParam, RepresentativeSurface,
    SurfaceFill, SurfaceFillParams, SurfaceFillPattern,
};

use crate::{
    SliceError, project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
};

use self::params::LayerContext;

pub(in crate::project_slice) fn group_fills_base(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<BaseGroupedFills, SliceError> {
    let horizontal = &prepared.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let record = &horizontal.objects[object_index].records[layer_index];
    let input = &prelude.records[layer_index];
    match (record, input) {
        (Some(record), Some(input)) => group_present_layer(LayerContext::new(
            prepared,
            prelude,
            record,
            input,
            layer_index,
        )),
        (None, None) => Ok(BaseGroupedFills::empty()),
        _ => unreachable!("validated fill-grouping record slots remain aligned"),
    }
}

fn group_present_layer(context: LayerContext<'_>) -> Result<BaseGroupedFills, SliceError> {
    let projected = params::project_layer(context)?;
    let mut grouped = coalesce::coalesce(projected);
    priority::apply(&mut grouped.surface_fills)?;
    Ok(grouped)
}
