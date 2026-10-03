/// `GCodeProcessor::process_G1` / `UsedFilaments::process_total_volume_cache`.
/// Track the global E
/// position (M82/M83, G90/G91, G92 resets), classify each move
/// (`move_type`, `GCodeProcessor.cpp:3834-3851`) and count only Extrude
/// moves — E-positive with an X or Y displacement. Retracts, unretracts
/// and E-only bowden re-primes do not count.
pub(in crate::project_slice::gcode_emit) fn account_volumes(
    gcode: &[u8],
    diameters: &[crate::options::OrcaFloat],
) -> Vec<f64> {
    let mut volumes = vec![0.0; diameters.len()];
    let mut filament = None;
    let text = std::str::from_utf8(gcode).unwrap_or_default();
    let mut e_position = 0.0_f64;
    let mut x_position = 0.0_f64;
    let mut y_position = 0.0_f64;
    let mut e_local_relative = false;
    let mut global_relative = false;
    let mut used = 0.0_f64;
    for line in text.lines() {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        let mut words = code.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        let letter_value = |letter: char| {
            words.clone().find_map(|word| {
                word.strip_prefix(letter)
                    .and_then(|value| value.parse::<f64>().ok())
                    .map(|value| f64::from(value as f32))
            })
        };
        if let Some(next) =
            super::super::processor::filament_change::selected_filament(code, diameters.len())
        {
            if let Some(previous) = filament
                && previous != next
            {
                volumes[previous] += used;
                used = 0.0;
            }
            filament = Some(next);
        }
        match command {
            "M82" => e_local_relative = false,
            "M83" => e_local_relative = true,
            "G90" => global_relative = false,
            "G91" => global_relative = true,
            "G92" => {
                if let Some(value) = letter_value('E') {
                    e_position = value;
                }
            }
            "G0" | "G1" | "G2" | "G3" => {
                let x = letter_value('X');
                let y = letter_value('Y');
                let e = letter_value('E');
                let start = (x_position, y_position, e_position);
                let e_relative = global_relative || e_local_relative;
                if let Some(value) = x {
                    x_position = apply_axis(x_position, value, global_relative);
                }
                if let Some(value) = y {
                    y_position = apply_axis(y_position, value, global_relative);
                }
                if let Some(value) = e {
                    e_position = apply_e(e_position, value, e_relative);
                }
                let dx = x_position - start.0;
                let dy = y_position - start.1;
                let delta_e = e_position - start.2;
                if delta_e > 0.0 && (dx != 0.0 || dy != 0.0) {
                    let diameter = diameters[filament.unwrap_or(0)].0 as f32;
                    let radius = 0.5_f32 * diameter;
                    let area = std::f32::consts::PI * (radius * radius);
                    // GCodeProcessor::process_G1 stores each extruded volume as float.
                    used += f64::from((f64::from(area) * delta_e) as f32);
                }
            }
            _ => {}
        }
    }
    volumes[filament.unwrap_or(0)] += used;
    volumes
}

fn apply_e(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}

fn apply_axis(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}
