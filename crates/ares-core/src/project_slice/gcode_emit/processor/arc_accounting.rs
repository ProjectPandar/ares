use super::motion::MotionState;
use super::motion::arc::{arc_discretization_steps, parse_arc};
use super::motion_util::word;

/// Upstream discretizes G2/G3 into `segments` internal G1s during
/// processing (`GCodeProcessor::process_G2_G3`), each incrementing the g1
/// line counter; the FILE still carries the single arc line, so the export
/// advances the counter by `1 + internal`. The geometry and the tolerance
/// branch come from the ONE shared derivation (`motion::arc::parse_arc` +
/// `arc_discretization_steps`) so the id accounting can never disagree
/// with the block-generating segment count.
pub(super) fn arc_internal_g1_lines(code: &str, command: &str, state: &MotionState) -> usize {
    let start = state.position;
    let (end_x, end_y) = (word(code, 'X'), word(code, 'Y'));
    let end_x = end_x.map_or(start[0], |value| {
        if state.relative {
            start[0] + value
        } else {
            value
        }
    });
    let end_y = end_y.map_or(start[1], |value| {
        if state.relative {
            start[1] + value
        } else {
            value
        }
    });
    // Degenerate R arc (coincident endpoints): `center_from_radius`
    // returns None and upstream's process_G2_G3 falls through without
    // segments.
    let Some(parsed) = parse_arc(
        command,
        code,
        start,
        [end_x, end_y, start[2]],
        state.relative,
    ) else {
        return 0;
    };
    let angle = parsed.sweep.abs().min(std::f64::consts::TAU);
    let feedrate_mm_s = word(code, 'F').map_or(state.feedrate, |value| {
        f64::from(value as f32 * super::motion_util::MMMIN_TO_MMSEC)
    }) as f32;
    let segments = if state.gcode_flavor == crate::GCodeFlavor::MarlinFirmware {
        // `plan_arc` (`GCodeProcessor.cpp:1690-1700`)
        const MAX_ARC_DEVIATION: f32 = 0.02;
        const MIN_ARC_SEGMENTS_PER_SEC: f32 = 50.0;
        const MIN_ARC_SEGMENT_MM: f32 = 0.1;
        const MAX_ARC_SEGMENT_MM: f32 = 2.0;
        let radius_mm = parsed.radius as f32;
        let segment_mm = ((8.0 * radius_mm * MAX_ARC_DEVIATION)
            .sqrt()
            .min(feedrate_mm_s / MIN_ARC_SEGMENTS_PER_SEC))
        .clamp(MIN_ARC_SEGMENT_MM, MAX_ARC_SEGMENT_MM);
        let flat_mm = radius_mm * angle as f32;
        ((flat_mm / segment_mm + 0.8) as usize).max(1)
    } else {
        arc_discretization_steps(parsed.radius, angle, 0.0125).max(1)
    };
    segments.saturating_sub(1)
}
