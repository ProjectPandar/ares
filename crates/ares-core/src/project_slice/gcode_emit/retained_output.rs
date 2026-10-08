use crate::{
    ProcessedGCode, SliceError, gcode_processor::printable_area::PrintableAreaTracker,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

/// OrcaSlicer.cpp:5981-5987,6215-6227: applied config and retained export.
pub(super) fn check(
    processed: ProcessedGCode,
    printable_area: PrintableAreaTracker,
    traversal: &PreparedPostClassicTraversal,
) -> Result<ProcessedGCode, SliceError> {
    let settings = &traversal.resolved.views.full;
    let error_code = printable_area.check(settings)?;
    if error_code != 0 {
        return Err(SliceError::ToolpathOutsidePrintableArea {
            error_code,
            gcode: processed.gcode,
            // bbs_3mf.cpp:2282 maps plater_id to index + 1; this view is selected.
            plate_index: crate::project_slice::state::first_plate_id(&traversal.project),
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        });
    }
    Ok(processed)
}
