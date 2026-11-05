use super::{PreparedPostClassicTraversal, header, motion};

pub(super) fn finalize_layer_count(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    state: &motion::EmitState,
) {
    // `GCodeProcessor.cpp:2920-2926,3323-3325,1136-1139`: count exact
    // reserved comment lines in the complete stream, including end templates.
    if header::enabled(traversal) {
        let layer_count = output
            .split(|byte| *byte == b'\n')
            .filter(|line| {
                line.strip_suffix(b"\r").unwrap_or(line) == state.tags.layer_change().as_bytes()
            })
            .count();
        header::finalize_layer_count(output, layer_count);
    }
}
