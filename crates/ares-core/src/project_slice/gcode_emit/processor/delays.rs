use super::motion_util::word;

pub(super) fn command_delay(code: &str) -> Option<f64> {
    // GCodeProcessor.cpp:5430–5436: SYNC T synchronizes with additional seconds.
    if code.split_whitespace().next() == Some("SYNC") {
        return word(code, 'T');
    }
    // GCodeProcessor.cpp:5347-5355: valid S short-circuits the P lookup.
    if code.starts_with("M400") {
        if let Some(seconds) = word(code, 'S') {
            return Some(seconds);
        }
        if let Some(milliseconds) = word(code, 'P') {
            return Some(milliseconds * 0.001);
        }
    }
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:4848-4856. process_G4's
    // `has_value('S') || has_value('P')` short-circuits, so a parseable S
    // suppresses the P lookup; otherwise P counts as milliseconds.
    if code.starts_with("G4") {
        if let Some(seconds) = word(code, 'S') {
            return Some(seconds);
        }
        if let Some(milliseconds) = word(code, 'P') {
            return Some(milliseconds * 0.001);
        }
    }
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:6367-6375.
    if code.starts_with("G29") && !code.starts_with("G29.") {
        return Some(260.0);
    }
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5150-5157.
    if code.starts_with("M191") && word(code, 'S').unwrap_or(0.0) > 40.0 {
        return Some(720.0);
    }
    None
}
