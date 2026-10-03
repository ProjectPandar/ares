//! `MotionOptions` field assembly split from `build.rs` (400-LOC rule).

mod assemble;
mod hilbert;
mod retraction;
mod wipe;

pub(super) use assemble::assemble;
