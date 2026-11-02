//! `GCodeProcessor::process_gcode_line:2911–2923` comment dispatch and
//! `process_VG1:4237–4546` endpoint/block admission without a move vertex.
use super::motion::{MotionBlock, MotionState};
use super::motion_util::{MMMIN_TO_MMSEC, word};

pub(super) fn command(line: &str) -> Option<&str> {
    let candidate = match line.strip_prefix(';') {
        Some(comment) if comment.starts_with(['V', 'v']) => comment,
        Some(_) => return None,
        None => line,
    };
    let code = candidate.split(';').next().unwrap_or_default().trim();
    (code.split_whitespace().next() == Some("VG1")).then_some(code)
}

impl MotionState {
    pub(super) fn virtual_motion(&mut self, code: &str) -> Option<MotionBlock> {
        let start = [
            self.position[0],
            self.position[1],
            self.position[2],
            self.e_position,
        ];
        let mut end = start;
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            let Some(value) = word(code, letter) else {
                continue;
            };
            let value = f64::from(value as f32 * self.units);
            let relative = self.relative || (axis == 3 && self.e_relative);
            end[axis] = if relative {
                start[axis] + value
            } else {
                self.origin[axis] + value
            };
        }
        self.position = [end[0], end[1], end[2]];
        self.e_position = end[3];
        if let Some(feedrate) = word(code, 'F') {
            self.feedrate = f64::from(feedrate as f32 * MMMIN_TO_MMSEC);
        }
        let delta = std::array::from_fn(|axis| end[axis] - start[axis]);
        self.segment_block(delta).map(|mut block| {
            block.virtual_motion = true;
            block
        })
    }
}
