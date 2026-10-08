//! GCode.cpp:3637-3645 PlaceholderParserError and CLI catch metadata.

use crate::{
    SliceError, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn from_failed_templates(
    failed_templates: &[(String, String)],
    traversal: &PreparedPostClassicTraversal,
) -> SliceError {
    let mut message = String::from("Failed to generate G-code for invalid custom G-code.\n\n");
    for (name, error) in failed_templates {
        message.push_str(name);
        message.push(' ');
        message.push_str(error);
        message.push('\n');
    }
    message.push_str("Please check the custom G-code or use the default custom G-code.");
    let settings = &traversal.resolved.views.full;
    SliceError::CustomGCode {
        message,
        plate_index: crate::project_slice::state::first_plate_id(&traversal.project),
        layer_height: f64::from(settings.process.object.layer_height.0 as f32),
        wall_loops: settings.process.region.wall_loops.0,
        sparse_infill_density: f64::from(settings.process.region.sparse_infill_density.0 as f32),
    }
}
