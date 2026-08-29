use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

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
    let count = activate.len().max(during_active.len()).max(speeds.len());
    let mut speed: Option<i32> = None;
    for index in 0..count {
        let on = activate[index.min(activate.len() - 1)].0
            && during_active[index.min(during_active.len() - 1)].0;
        if on {
            let value = speeds[index.min(speeds.len() - 1)].0;
            speed = Some(speed.map_or(value, |current| current.max(value)));
        }
    }
    if let Some(speed) = speed {
        let pwm = (f64::from(speed) / 100.0 * 255.0) as i32;
        output.extend_from_slice(format!("M106 P3 S{pwm}\n").as_bytes());
    }
}
