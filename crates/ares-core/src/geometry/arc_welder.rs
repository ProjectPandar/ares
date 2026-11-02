/// `ArcWelder::arc_discretization_steps` (`ArcWelder.hpp:48-64`) — the
/// non-MarlinFirmware segment count at the 0.0125mm gcode tolerance.
pub(crate) fn arc_discretization_steps(radius: f64, angle: f64, deviation: f64) -> usize {
    let distance = radius - deviation;
    if distance < 1.0e-4 {
        if angle < std::f64::consts::PI
            || radius * (1.0 + (std::f64::consts::PI - 0.5 * angle).cos()) < deviation
        {
            1
        } else {
            2
        }
    } else {
        (angle / (2.0 * (distance / radius).acos())).ceil() as usize
    }
}
