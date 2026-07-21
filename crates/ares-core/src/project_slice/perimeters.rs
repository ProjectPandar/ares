use crate::{
    Project, SliceError, geometry::CoordinateScale,
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::compensation::{PreparedPostCompensation, prepare_post_compensation};
use context::prepare_perimeter_contexts;
use preflight::preflight_perimeter_flows;
use types::PostPerimeterInputPrintObject;

pub(super) mod context;
pub(super) mod flow;
pub(super) mod preflight;
pub(super) mod types;

pub(super) struct PreparedPostPerimeterInputs {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) objects: Vec<PostPerimeterInputPrintObject>,
}

pub(super) fn prepare_post_perimeter_inputs(
    project: impl AsRef<[u8]>,
) -> Result<PreparedPostPerimeterInputs, SliceError> {
    finish_post_perimeter_inputs(prepare_post_compensation(project)?)
}

pub(super) fn finish_post_perimeter_inputs(
    prepared: PreparedPostCompensation,
) -> Result<PreparedPostPerimeterInputs, SliceError> {
    let initial_layer_width = prepared
        .resolved
        .views
        .full
        .process
        .print
        .initial_layer_line_width;
    let nozzle_diameters = &prepared.resolved.views.full.project.print.nozzle_diameter;
    let spiral_mode = prepared.resolved.views.full.process.print.spiral_mode.0;
    let flows = preflight_perimeter_flows(
        &prepared.objects,
        &prepared.resolved.objects,
        initial_layer_width,
        nozzle_diameters,
    )?;
    let PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepared;
    let objects = prepare_perimeter_contexts(objects, flows, &resolved.objects, spiral_mode);
    Ok(PreparedPostPerimeterInputs {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}
