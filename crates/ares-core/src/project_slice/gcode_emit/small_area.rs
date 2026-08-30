use crate::{
    SliceError, extrusions::SmallAreaInfillFlowCompensation,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn from_traversal(
    traversal: &PreparedPostClassicTraversal,
) -> Result<SmallAreaInfillFlowCompensation, SliceError> {
    let first_region = traversal
        .resolved
        .objects
        .first()
        .and_then(|object| object.layer_candidates.first())
        .and_then(|layer| layer.model_parts.first())
        .map(|part| &part.region);
    let enabled = first_region.map_or(
        traversal
            .resolved
            .views
            .full
            .process
            .region
            .small_area_infill_flow_compensation
            .0,
        |region| region.small_area_infill_flow_compensation.0,
    );
    if !enabled {
        return Ok(SmallAreaInfillFlowCompensation::disabled());
    }
    SmallAreaInfillFlowCompensation::parse(
        traversal
            .resolved
            .views
            .full
            .process
            .gcode
            .small_area_infill_flow_compensation_model
            .0
            .clone(),
        true,
        true,
        true,
    )
}
