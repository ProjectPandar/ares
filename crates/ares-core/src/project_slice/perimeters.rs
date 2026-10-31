use crate::{
    Project, SliceError, geometry::CoordinateScale,
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::ProjectBytes;
use super::compensation::{PreparedPostCompensation, prepare_post_compensation};
use context::prepare_perimeter_contexts;

pub(super) mod arachne;
pub(super) mod classic;
pub(super) mod layer_region;
use preflight::preflight_perimeter_flows;
use types::PostPerimeterInputPrintObject;

pub(super) mod context;
pub(super) mod flow;
mod groups;
pub(super) mod preflight;
pub(super) mod types;

pub(super) struct PreparedPostPerimeterInputs {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) objects: Vec<PostPerimeterInputPrintObject>,
}

pub(super) fn prepare_post_perimeter_inputs<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<PreparedPostPerimeterInputs, SliceError> {
    finish_post_perimeter_inputs(prepare_post_compensation(project.into_source())?)
}

pub(super) fn prepare_post_classic_prelude<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicPrelude, SliceError> {
    classic::finish_classic_prelude(prepare_post_perimeter_inputs(project.into_source())?)
}

fn classic_infill_boundary(
    prelude: classic::PreparedPostClassicPrelude,
    context: &mut crate::SlicerContext,
) -> Result<classic::PreparedPostClassicInfillBoundary, SliceError> {
    let top_split = classic::finish_classic_top_split(prelude)?;
    let onion = classic::finish_classic_onion(top_split)?;
    let hierarchy = classic::finish_classic_hierarchy(onion);
    let traversal = classic::finish_classic_traversal(hierarchy)?;
    let raw_paths = classic::finish_classic_raw_paths(Box::new(traversal), context)?;
    let chained_loops = classic::finish_classic_chained_loops(raw_paths);
    let entity_collections = classic::finish_classic_entity_collections(chained_loops);
    let perimeter_append = classic::finish_classic_perimeter_append(entity_collections);
    let gap_domain = classic::finish_classic_gap_domain(perimeter_append)?;
    let medial_gap = classic::finish_classic_medial_gap(gap_domain)?;
    let gap_extrusion = classic::finish_classic_gap_extrusion(medial_gap)?;
    classic::finish_classic_infill_boundary(gap_extrusion)
}

pub(super) fn prepare_post_layer_region_perimeters<'a>(
    project: impl ProjectBytes<'a>,
    context: &mut crate::SlicerContext,
) -> Result<layer_region::PreparedPostLayerRegionPerimeters, SliceError> {
    // `LayerRegion::make_perimeters` dispatches `process_arachne` per region
    // (`LayerRegion.cpp:104-115`); the classic chain remains the shared
    // context spine and arachne records materialize through
    // `PerimeterGenerator::process_arachne` (`PerimeterGenerator.cpp:2093`).
    let prelude = prepare_post_classic_prelude(project)?;
    let arachne = arachne::materialize::finish(&prelude, context)?;
    let has_arachne = arachne
        .objects
        .iter()
        .any(|object| object.records.iter().any(Option::is_some));
    let infill_boundary = classic_infill_boundary(prelude, context)?;
    let mut prepared =
        layer_region::finish_with_arachne(infill_boundary, has_arachne.then_some(arachne));
    for (source, output) in prepared
        .predecessor
        .as_classic()
        .objects
        .iter()
        .zip(&mut prepared.objects)
    {
        let input = &source
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        groups::redistribute::apply(input, output)?;
    }
    Ok(prepared)
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
    let objects = prepare_perimeter_contexts(objects, flows, &resolved.objects, spiral_mode)?;
    Ok(PreparedPostPerimeterInputs {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}
