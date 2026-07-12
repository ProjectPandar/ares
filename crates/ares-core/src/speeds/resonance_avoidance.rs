use crate::{ExtrusionMove, PrintPathRole, SpeedOptions, ToolpathMoveKind};

pub(super) fn adjusted_speed(
    options: &SpeedOptions,
    move_: &ExtrusionMove,
    speed_mm_s: f64,
) -> f64 {
    if !options.resonance_avoidance()
        || move_.kind() != ToolpathMoveKind::Print
        || move_.role() != PrintPathRole::ExternalPerimeter
    {
        return speed_mm_s;
    }

    let max = options.max_resonance_avoidance_speed_mm_s();
    if speed_mm_s >= max {
        return speed_mm_s;
    }

    let min = options.min_resonance_avoidance_speed_mm_s();
    let midpoint = min + ((max - min) / 2.0);
    if speed_mm_s < midpoint {
        speed_mm_s.min(min)
    } else {
        max
    }
}
