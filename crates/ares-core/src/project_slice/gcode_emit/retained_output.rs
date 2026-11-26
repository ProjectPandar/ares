use crate::{
    SliceError, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

/// OrcaSlicer.cpp:5981-5987,6215-6227: applied config and retained export.
pub(super) fn outside_printable_area(
    error_code: u8,
    gcode: Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
) -> SliceError {
    let settings = &traversal.resolved.views.full;
    SliceError::ToolpathOutsidePrintableArea {
        error_code,
        gcode,
        // bbs_3mf.cpp:2282 maps plater_id to index + 1; this view is selected.
        plate_index: traversal.project.first_plate_id(),
        layer_height: f64::from(settings.process.object.layer_height.0 as f32),
        wall_loops: settings.process.region.wall_loops.0,
        sparse_infill_density: f64::from(settings.process.region.sparse_infill_density.0 as f32),
    }
}
