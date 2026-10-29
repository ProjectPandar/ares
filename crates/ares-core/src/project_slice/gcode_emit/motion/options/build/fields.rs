//! `MotionOptions` field assembly split from `build.rs` (400-LOC rule).

mod assemble;
pub(in crate::project_slice::gcode_emit::motion::options::build) mod hilbert;
mod retraction;
mod wipe;

pub(super) use assemble::assemble;
