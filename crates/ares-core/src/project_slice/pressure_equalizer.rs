//! PressureEqualizer — a per-layer G-code text post-pass that smooths the
//! volumetric extrusion rate slope (`GCode/PressureEqualizer.cpp`, 955 LOC).
//!
//! Slice 1 of the port: the G-code line parser (`process_line`,
//! `PressureEqualizer.cpp:272-455`) and the line model (`GCodeLine`,
//! `PressureEqualizer.hpp:110-160`). Pure parsing + classification; the
//! rate limiter (`adjust_volumetric_rate`) and the re-emission
//! (`output_gcode_line`/`push_line_to_output`) land in later slices.

pub(crate) const EXTRUSION_ROLE_TAG: &str = ";_EXTRUSION_ROLE:";
pub(crate) const EXTRUDE_END_TAG: &str = ";_EXTRUDE_END";
pub(crate) const EXTRUDE_SET_SPEED_TAG: &str = ";_EXTRUDE_SET_SPEED";
pub(crate) const EXTERNAL_PERIMETER_TAG: &str = ";_EXTERNAL_PERIMETER";

/// `max_look_back_limit` (`PressureEqualizer.cpp:64`).
pub(crate) const MAX_LOOK_BACK_LIMIT: usize = 128;

/// `max_ignored_gap_between_extruding_segments` (`PressureEqualizer.cpp:70`).
pub(crate) const MAX_IGNORED_GAP_BETWEEN_EXTRUDING_SEGMENTS: f64 = 3.0;

mod limiter;
mod line;
mod tests;

pub(crate) use line::{GCodeLine, GCodeLineType, LineParser};
