mod coalesce;
mod narrow;
mod params;
mod priority;
mod types;

pub(in crate::project_slice) use params::{reset_rand_state, template_rotation_angle};
pub(in crate::project_slice) use types::{
    GroupedFills, LockDensityParam, LockFlowParam, LockRegionParam, RepresentativeSurface,
    SurfaceFill, SurfaceFillParams, SurfaceFillPattern,
};

use crate::{
    SliceError, geometry::ClipperError,
    project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
};

use self::params::LayerContext;
use crate::project_slice::perimeters::types::RegionLayerIndex;

pub(in crate::project_slice) fn group_fills(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<GroupedFills, SliceError> {
    let horizontal = &prepared.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let mut contexts = Vec::with_capacity(post_regions.regions.len());
    for region_index in 0..post_regions.regions.len() {
        let slot = prelude.region_layer_record_index(RegionLayerIndex {
            region_index,
            layer_index,
        });
        let record = &horizontal.objects[object_index].records[slot];
        let input = &prelude.records[slot];
        match (record, input) {
            (Some(record), Some(input)) => contexts.push(LayerContext::new(
                prepared,
                prelude,
                record,
                input,
                layer_index,
            )),
            (None, None) => {}
            _ => unreachable!("validated fill-grouping record slots remain aligned"),
        }
    }
    group_present_layer(&contexts)
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned(),
        ),
    }
}

fn group_present_layer(contexts: &[LayerContext<'_>]) -> Result<GroupedFills, SliceError> {
    let Some(context) = contexts.first() else {
        return Ok(GroupedFills::empty());
    };
    let narrow = narrow::Context {
        enabled: context.object.detect_narrow_internal_solid_infill.0,
        layer_id: context.planned.id,
        scale: context.scale,
    };
    let (projected, lock_region_param) = params::project_layers(contexts)?;
    let mut grouped = GroupedFills {
        surface_fills: coalesce::coalesce(&projected)?,
        lock_region_param,
    };

    priority::apply(&mut grouped.surface_fills)?;
    narrow::apply(&mut grouped.surface_fills, narrow)?;
    Ok(grouped)
}
