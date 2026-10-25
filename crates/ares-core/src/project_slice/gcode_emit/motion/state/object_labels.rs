use super::EmitState;

pub(in crate::project_slice::gcode_emit) fn queue_object_start(
    state: &mut EmitState,
    label_id: u32,
    encoded_labels: [u8; 12],
) {
    state.pending_object_start = Some((label_id, encoded_labels));
}

pub(in crate::project_slice::gcode_emit) fn queue_exclude_start(
    state: &mut EmitState,
    text: String,
) {
    state.pending_exclude_start = Some(text);
}

/// Mirrors `GCode.cpp:5478-5494`: an unflushed start label is dropped
/// (empty instance) and only then does the end label get armed.
pub(in crate::project_slice::gcode_emit) fn queue_exclude_end(state: &mut EmitState, text: String) {
    if state.pending_exclude_start.take().is_none() {
        state.pending_exclude_end = Some(text);
    }
}

/// Mirrors `GCode.cpp:5478-5494` for the BBL label: an unflushed start
/// label is dropped (empty instance) and only then does the end label
/// get armed.
pub(in crate::project_slice::gcode_emit) fn queue_object_stop_label(
    state: &mut EmitState,
    label_id: u32,
) {
    if state.pending_object_start.take().is_none() {
        state.pending_object_stop_label = Some(label_id);
    }
}

pub(in crate::project_slice::gcode_emit) fn append_exclude_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if let Some(label_id) = state.pending_object_stop_label.take() {
        output.extend_from_slice(
            format!("; stop printing object, unique label id: {label_id}\nM625\n").as_bytes(),
        );
    }
    if let Some(text) = state.pending_exclude_end.take() {
        output.extend_from_slice(text.as_bytes());
        if !state.options.use_relative_e_distances && text.lines().any(|line| line == "G92 E0") {
            state.extruder.e_position = 0.0;
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn append_object_start(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    // `add_object_change_labels` writes the end label before the start label
    // (`GCodeWriter.cpp:1197-1201`).
    append_exclude_end(output, state);
    if let Some((label_id, encoded_labels)) = state.pending_object_start.take() {
        output.extend_from_slice(
            format!("; start printing object, unique label id: {label_id}\nM624 ").as_bytes(),
        );
        output.extend_from_slice(&encoded_labels);
        output.push(b'\n');
    }
    if let Some(text) = state.pending_exclude_start.take() {
        if !state.options.use_relative_e_distances && state.extruder.e_position.abs() > f64::EPSILON
        {
            output.extend_from_slice(b"G92 E0\n");
            state.extruder.e_position = 0.0;
        }
        output.extend_from_slice(text.as_bytes());
    }
}
