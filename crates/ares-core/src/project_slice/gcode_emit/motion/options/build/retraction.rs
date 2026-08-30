pub(super) fn speeds(gcode: &crate::GCodeOptions) -> (f64, f64) {
    let retraction = gcode
        .retraction_speed
        .0
        .first()
        .map_or(0.0, |value| value.0)
        .round();
    let configured_deretraction = gcode
        .deretraction_speed
        .0
        .first()
        .map_or(0.0, |value| value.0)
        .round();
    let deretraction = if configured_deretraction > 0.0 {
        configured_deretraction
    } else {
        retraction
    };
    (retraction, deretraction)
}
