use super::super::perimeters::classic::traversal::PreparedPostClassicTraversal;
use super::{header, layers::Schedule, tags::Tags};
use crate::{SliceError, SlicerContext, gcode_thumbnails};

/// `GCode.cpp:4233-4334`: header, BBL config or compatible thumbnails, widths.
pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    schedule: &Schedule,
    export_config: &[u8],
    context: &SlicerContext<'_>,
) -> Result<Tags, SliceError> {
    let max_print_z = if traversal
        .resolved
        .views
        .full
        .printer
        .remaining
        .belt_printer
        .0
    {
        // GCode.cpp:4280–4282 reads the last physical layer, before belt
        // collection removes empty trailing layers from the export schedule.
        traversal
            .objects
            .iter()
            .filter_map(|object| object.physical_layers().last().map(|layer| layer.print_z))
            .fold(-1.0, f64::max)
    } else {
        schedule.merged.last().map_or(0.0, |entry| entry.0)
    };
    let tags = Tags::of(traversal);
    if header::enabled(traversal) {
        header::append_header(output, traversal, schedule, max_print_z);
    }
    // GCode.cpp:4285-4300/4882-4935: BBL keeps config up front;
    // non-BBL writes it after executable output and before statistics.
    if tags.is_bbl() {
        output.extend_from_slice(export_config);
    }
    if !tags.is_bbl() {
        let settings = &traversal.resolved.views.full;
        let definitions = settings.printer.remaining.thumbnails.as_str();
        let images = context.thumbnail_images();
        if context.render_thumbnails && images.is_empty() {
            gcode_thumbnails::append_rendered(
                output,
                &traversal.project,
                settings,
                context.cancel_ref(),
            )?;
        } else {
            gcode_thumbnails::append(output, definitions, images)?;
        }
    }
    header::append_width_block(output, traversal)?;
    Ok(tags)
}
