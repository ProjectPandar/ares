use super::super::perimeters::classic::traversal::PreparedPostClassicTraversal;
use super::{header, layers::Schedule, tags::Tags};
use crate::{SliceError, ThumbnailData};

/// `GCode.cpp:4233-4334`: header, BBL config or compatible thumbnails, widths.
pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    schedule: &Schedule,
    export_config: &[u8],
    images: &[ThumbnailData],
) -> Result<Tags, SliceError> {
    let max_print_z = schedule.merged.last().map_or(0.0, |entry| entry.0);
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
        crate::gcode_thumbnails::append(
            output,
            traversal
                .resolved
                .views
                .full
                .printer
                .remaining
                .thumbnails
                .as_str(),
            images,
        )?;
    }
    header::append_width_block(output, traversal)?;
    Ok(tags)
}
