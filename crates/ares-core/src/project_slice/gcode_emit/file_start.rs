use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint::FirstLayerBounds, placeholders, processor, template, value::Value};

/// `file_start_gcode` is rendered before every generated header
/// (`GCode.cpp:2551-2566`). Timing and filament totals are resolved by the
/// post-processor after the complete file is available.
pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<FirstLayerBounds>,
) -> Result<(), SliceError> {
    let source = &traversal.resolved.views.runtime_gcode.file_start_gcode.0;
    if source.is_empty() {
        return Ok(());
    }
    let mut config = placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    config.insert(
        "print_time_sec",
        Value::String(processor::PRINT_TIME_SEC_PLACEHOLDER.to_owned()),
    );
    config.insert(
        "used_filament_length",
        Value::String(processor::USED_FILAMENT_LENGTH_PLACEHOLDER.to_owned()),
    );
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
