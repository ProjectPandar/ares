use crate::{
    Project, SliceError, geometry::CoordinateScale,
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::compensation::{PreparedPostCompensation, prepare_post_compensation};
use context::prepare_perimeter_contexts;

pub(super) mod classic;
pub(super) mod layer_region;
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

pub(super) fn prepare_post_classic_prelude(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicPrelude, SliceError> {
    classic::finish_classic_prelude(prepare_post_perimeter_inputs(project)?)
}

pub(super) fn prepare_post_classic_top_split(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicTopSplit, SliceError> {
    classic::finish_classic_top_split(prepare_post_classic_prelude(project)?)
}

pub(super) fn prepare_post_classic_onion(
    project: impl AsRef<[u8]>,
) -> Result<Box<classic::PreparedPostClassicOnion>, SliceError> {
    Ok(Box::new(classic::finish_classic_onion(
        prepare_post_classic_top_split(project)?,
    )?))
}

pub(super) fn prepare_post_classic_hierarchy(
    project: impl AsRef<[u8]>,
) -> Result<Box<classic::PreparedPostClassicHierarchy>, SliceError> {
    Ok(Box::new(classic::finish_classic_hierarchy(
        *prepare_post_classic_onion(project)?,
    )))
}

pub(super) fn prepare_post_classic_traversal(
    project: impl AsRef<[u8]>,
) -> Result<Box<classic::PreparedPostClassicTraversal>, SliceError> {
    Ok(Box::new(classic::finish_classic_traversal(
        *prepare_post_classic_hierarchy(project)?,
    )))
}

pub(super) fn prepare_post_classic_raw_paths(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicRawPaths, SliceError> {
    classic::finish_classic_raw_paths(prepare_post_classic_traversal(project)?)
}

pub(super) fn prepare_post_classic_chained_loops(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicChainedLoops, SliceError> {
    Ok(classic::finish_classic_chained_loops(
        prepare_post_classic_raw_paths(project)?,
    ))
}

pub(super) fn prepare_post_classic_entity_collections(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicEntityCollections, SliceError> {
    Ok(classic::finish_classic_entity_collections(
        prepare_post_classic_chained_loops(project)?,
    ))
}

pub(super) fn prepare_post_classic_perimeter_append(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicPerimeterAppend, SliceError> {
    Ok(classic::finish_classic_perimeter_append(
        prepare_post_classic_entity_collections(project)?,
    ))
}

pub(super) fn prepare_post_classic_gap_domain(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicGapDomain, SliceError> {
    classic::finish_classic_gap_domain(prepare_post_classic_perimeter_append(project)?)
}

pub(super) fn prepare_post_classic_medial_gap(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicMedialGap, SliceError> {
    classic::finish_classic_medial_gap(prepare_post_classic_gap_domain(project)?)
}

pub(super) fn prepare_post_classic_gap_extrusion(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicGapExtrusion, SliceError> {
    classic::finish_classic_gap_extrusion(prepare_post_classic_medial_gap(project)?)
}

pub(super) fn prepare_post_classic_infill_boundary(
    project: impl AsRef<[u8]>,
) -> Result<classic::PreparedPostClassicInfillBoundary, SliceError> {
    classic::finish_classic_infill_boundary(prepare_post_classic_gap_extrusion(project)?)
}

pub(super) fn prepare_post_layer_region_perimeters(
    project: impl AsRef<[u8]>,
) -> Result<layer_region::PreparedPostLayerRegionPerimeters, SliceError> {
    Ok(layer_region::finish(prepare_post_classic_infill_boundary(
        project,
    )?))
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
