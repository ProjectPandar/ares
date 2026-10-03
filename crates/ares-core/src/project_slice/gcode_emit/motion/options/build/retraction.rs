pub(super) fn speeds(gcode: &crate::GCodeOptions, filament: usize) -> (f64, f64) {
    let retraction = gcode
        .retraction_speed
        .0
        .get(filament)
        .or_else(|| gcode.retraction_speed.0.first())
        .map_or(0.5, |value| value.0 + 0.5)
        .floor();
    let configured_deretraction = gcode
        .deretraction_speed
        .0
        .get(filament)
        .or_else(|| gcode.deretraction_speed.0.first())
        .map_or(0.5, |value| value.0 + 0.5)
        .floor();
    let deretraction = if configured_deretraction > 0.0 {
        configured_deretraction
    } else {
        retraction
    };
    (retraction, deretraction)
}
