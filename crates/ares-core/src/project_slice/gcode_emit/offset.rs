use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

/// `GCode.cpp:8124-8128`: points are translated by the print origin and then
/// corrected by the active physical extruder offset.
pub(super) fn initial_extruder(traversal: &PreparedPostClassicTraversal) -> (f64, f64) {
    let settings = &traversal.resolved.views.full;
    let slot = settings
        .project
        .gcode
        .filament_map
        .0
        .first()
        .map_or(0, |value| value.0.saturating_sub(1).max(0) as usize);
    let heterogeneous = settings
        .printer
        .gcode
        .extruder_type
        .0
        .windows(2)
        .any(|pair| pair[0] != pair[1]);
    let physical = if heterogeneous {
        settings
            .printer
            .gcode
            .physical_extruder_map
            .0
            .get(slot)
            .and_then(|value| usize::try_from(value.0).ok())
            .unwrap_or(slot)
    } else {
        slot
    };
    let point = settings
        .project
        .print
        .extruder_offset
        .0
        .get(physical)
        .or_else(|| settings.project.print.extruder_offset.0.first());
    point.map_or((0.0, 0.0), |point| (point.x, point.y))
}
