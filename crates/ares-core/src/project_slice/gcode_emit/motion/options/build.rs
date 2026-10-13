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

    pub(in crate::project_slice::gcode_emit) fn apply_region_outer_wall_speed(
        &mut self,
        current: &crate::RegionOptions,
    ) {
        region::apply_outer_wall_speed(self, current);
    }
}
