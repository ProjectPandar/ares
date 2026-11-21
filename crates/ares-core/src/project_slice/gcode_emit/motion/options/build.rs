//! Builds `MotionOptions` from the resolved project configuration
//! (`GCodeWriter::apply_print_config` equivalents).

mod context;
mod crossing;
mod fields;
mod region;
mod retraction;
mod scarf;

use super::MotionOptions;

impl MotionOptions {
    pub(in crate::project_slice::gcode_emit) fn from_traversal(
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
        filament: usize,
    ) -> Self {
        fields::assemble(traversal, context::prelude(traversal, filament))
    }

    /// `GCode.cpp:6136/6163`: re-apply the whole region config before
    /// emitting that region's perimeter/infill entities — the same
    /// `region::apply` the initial build runs for the first region.
    pub(in crate::project_slice::gcode_emit) fn apply_region(
        &mut self,
        default: &crate::options::ProcessRegionSourceOptions,
        current: Option<&crate::RegionOptions>,
        wipe_travel_speed: f64,
        nozzle: f64,
    ) {
        region::apply(self, default, current, wipe_travel_speed, nozzle);
    }
}
