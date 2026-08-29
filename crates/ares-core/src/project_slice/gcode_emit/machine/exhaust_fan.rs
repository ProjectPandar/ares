use crate::{
    OrcaBool, OrcaInt, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

#[cfg(test)]
mod tests;

// GCode.cpp:3181-3196: when air filtration is supported and any extruder has
// `activate_air_filtration` + `activate_air_filtration_during_print`, upstream
// emits the during-print exhaust fan (`set_exhaust_fan`, M106 P3) right after
// machine_start_gcode at print start. Upstream applies no flavor guard here.
pub(super) fn append_print_start(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let views = &traversal.resolved.views;
    if !views.runtime_gcode.support_air_filtration.0 {
        return;
    }
    let filament = &views.full.filament.print;
    let activate = &filament.activate_air_filtration.0;
    let during_active = &filament.activate_air_filtration_during_print.0;
    let speeds = &filament.during_print_exhaust_fan_speed.0;
    if let Some(speed) = max_during_print_speed(activate, during_active, speeds) {
        let pwm = (f64::from(speed) / 100.0 * 255.0) as i32;
        output.extend_from_slice(format!("M106 P3 S{pwm}\n").as_bytes());
    }
}

fn max_during_print_speed(
    activate: &[OrcaBool],
    during_active: &[OrcaBool],
    speeds: &[OrcaInt],
) -> Option<i32> {
    let count = activate.len().max(during_active.len()).max(speeds.len());
    (0..count)
        .filter(|&index| {
            activate[index.min(activate.len() - 1)].0
                && during_active[index.min(during_active.len() - 1)].0
        })
        .map(|index| speeds[index.min(speeds.len() - 1)].0)
        .max()
}
