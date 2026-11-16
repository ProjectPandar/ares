use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint::FirstLayerBounds, placeholders, template, value::TemplateRng};

pub(super) struct FileStart<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<FirstLayerBounds>,
    pub(super) layer_count: usize,
    pub(super) template_rng: &'a TemplateRng,
}

/// `file_start_gcode` is rendered before every generated header
/// (`GCode.cpp:4230-4231`), through the regular custom G-code path, so it draws
/// from the same template stream as the templates emitted after it. Timing and
/// filament totals are resolved by the post-processor after the complete file
/// is available.
pub(super) fn append(output: &mut Vec<u8>, start: FileStart<'_>) -> Result<(), SliceError> {
    let source = &start
        .traversal
        .resolved
        .views
        .runtime_gcode
        .file_start_gcode
        .0;
    if source.is_empty() {
        return Ok(());
    }
    let mut config = placeholders::base_config(
        start.traversal,
        start.metadata,
        start.first_layer_bounds,
        Some(start.template_rng),
        start.layer_count,
    )?;
    let rendered = template::render(source, &mut config).map_err(|error| {
        SliceError::InvalidInput(format!(
            "invalid project file-start G-code template: {error}"
        ))
    })?;
    if !rendered.is_empty() {
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    Ok(())
}
