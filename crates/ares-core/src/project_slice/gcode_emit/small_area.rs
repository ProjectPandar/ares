use crate::{
    ProcessInfillPattern, SliceError, extrusions::SmallAreaInfillFlowCompensation,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn from_traversal(
    traversal: &PreparedPostClassicTraversal,
) -> Result<SmallAreaInfillFlowCompensation, SliceError> {
    let settings = &traversal.resolved.views.full.process;
    let model = &settings.gcode.small_area_infill_flow_compensation_model.0;
    if !settings.region.small_area_infill_flow_compensation.0 || model.is_empty() {
        return Ok(SmallAreaInfillFlowCompensation::disabled());
    }
    // GCode.cpp:2548-2549 constructs the global model; the current region
    // gate is evaluated separately at the segment emission boundary.
    SmallAreaInfillFlowCompensation::parse(model.clone(), true, true, true)
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::project_slice::gcode_emit) struct Gate {
    pub(in crate::project_slice::gcode_emit) enabled: bool,
    pub(in crate::project_slice::gcode_emit) bottom: ProcessInfillPattern,
    pub(in crate::project_slice::gcode_emit) internal: ProcessInfillPattern,
    pub(in crate::project_slice::gcode_emit) top: ProcessInfillPattern,
}

impl Gate {
    pub(in crate::project_slice::gcode_emit) fn needs(
        self,
        feature: &str,
        first_layer: bool,
    ) -> bool {
        let supported = |pattern| {
            matches!(
                pattern,
                ProcessInfillPattern::Rectilinear
                    | ProcessInfillPattern::AlignedRectilinear
                    | ProcessInfillPattern::Monotonic
                    | ProcessInfillPattern::MonotonicLine
            )
        };
        self.enabled
            && ((first_layer && supported(self.bottom))
                || (feature == "Internal solid infill" && supported(self.internal))
                || (feature == "Top surface" && supported(self.top)))
    }
}
