use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint::FirstLayerBounds, machine, motion::EmitState, placeholders, template};

pub(super) struct FileStart<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<FirstLayerBounds>,
    pub(super) layer_count: usize,
}

/// Render the buffered mode script before file start (`GCode.cpp:4221-4231`),
/// so both use the same context even though the mode script is written later.
pub(super) fn append(
    output: &mut Vec<u8>,
    start: FileStart<'_>,
    state: &mut EmitState,
) -> Result<String, SliceError> {
    let mode = state.template_context.mode_script().to_owned();
    let mode_gcode = machine::render_start_template(
        ("imex_mode_gcode", &mode),
        state,
        start.traversal,
        start.metadata,
        start.first_layer_bounds,
    )?;
    let source = &start
        .traversal
        .resolved
        .views
        .runtime_gcode
        .file_start_gcode
        .0;
    if source.is_empty() {
        return Ok(mode_gcode);
    }
    let mut config = placeholders::base_config(
        start.traversal,
        start.metadata,
        start.first_layer_bounds,
        Some(&state.template_rng),
        start.layer_count,
    )?;
    let rendered =
        template::render(source, &mut config, &mut state.template_context).map_err(|error| {
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
    Ok(mode_gcode)
}
