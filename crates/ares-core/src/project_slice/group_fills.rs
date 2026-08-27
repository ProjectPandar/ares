mod coalesce;
mod narrow;
mod params;
mod priority;
mod types;

pub(in crate::project_slice) use params::simple_rotation_angle;
pub(in crate::project_slice) use types::{
    GroupedFills, LockDensityParam, LockFlowParam, LockRegionParam, RepresentativeSurface,
    SurfaceFill, SurfaceFillParams, SurfaceFillPattern,
};

use crate::{
    SliceError, geometry::ClipperError,
    project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
};

use self::params::LayerContext;

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
        (None, None) => Ok(GroupedFills::empty()),
        _ => unreachable!("validated fill-grouping record slots remain aligned"),
    }
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("fill-grouping operations contain only closed polygon paths")
        }
    }
}

fn group_present_layer(context: LayerContext<'_>) -> Result<GroupedFills, SliceError> {
    let narrow = narrow::Context {
        enabled: context.object.detect_narrow_internal_solid_infill.0,
        layer_id: context.planned.id,
        scale: context.scale,
    };
    let projected = params::project_layer(context)?;
    let mut grouped = coalesce::coalesce(projected);
    priority::apply(&mut grouped.surface_fills)?;
    narrow::apply(&mut grouped.surface_fills, narrow)?;
    Ok(grouped)
}
