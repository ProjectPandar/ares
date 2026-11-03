use crate::{
    GCodeFlavor,
    project_slice::{extruders, perimeters::classic::traversal::PreparedPostClassicTraversal},
};

use super::super::temperature::append as append_nozzle_temperature;

// GCode.cpp:3994-4003,5497-5513,5525-5530: preserve each reader's filament context.
pub(super) fn first_layer_bed_temperature(
    traversal: &PreparedPostClassicTraversal,
    first_printing_filament: usize,
    first_layer_tools: &[usize],
) -> i32 {
    let settings = &traversal.resolved.views.full;
    use crate::ProjectBedType;
    let filament = &settings.filament.print;
    let temps: &[crate::OrcaInts] = match settings.project.print.curr_bed_type {
        ProjectBedType::DefaultPlate => &[],
        ProjectBedType::SupertackPlate => {
            std::slice::from_ref(&filament.supertack_plate_temp_initial_layer)
        }
        ProjectBedType::CoolPlate => std::slice::from_ref(&filament.cool_plate_temp_initial_layer),
        ProjectBedType::EngineeringPlate => {
            std::slice::from_ref(&filament.eng_plate_temp_initial_layer)
        }
        ProjectBedType::HighTempPlate => {
            std::slice::from_ref(&filament.hot_plate_temp_initial_layer)
        }
        ProjectBedType::TexturedPeiPlate => {
            std::slice::from_ref(&filament.textured_plate_temp_initial_layer)
        }
        ProjectBedType::TexturedCoolPlate => {
            std::slice::from_ref(&filament.textured_cool_plate_temp_initial_layer)
        }
    };
    let temps = temps
        .iter()
        .flat_map(|values| values.0.iter().map(|value| value.0))
        .collect::<Vec<_>>();
    match settings.printer.gcode.bed_temperature_formula {
        crate::BedTemperatureFormula::FirstFilament => temps
            .get(first_printing_filament)
            .or_else(|| temps.first())
            .copied()
            .unwrap_or(0),
        crate::BedTemperatureFormula::HighestTemp => first_layer_tools
            .iter()
            .filter_map(|&tool| temps.get(tool).or_else(|| temps.first()))
            .copied()
            .fold(0, i32::max),
    }
}

/// `_print_first_layer_bed_temperature` and
/// `_print_first_layer_extruder_temperatures` (`GCode.cpp:3118-3124`,
/// `GCode.cpp:4023-4087`). Temperatures precede the custom-role tag and are
/// omitted when the rendered machine-start G-code already controls them.
pub(super) fn append_startup(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    machine_start_gcode: &str,
    bed_temperature: i32,
) -> i32 {
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    if flavor == GCodeFlavor::Klipper {
        return 0;
    }

    if bed_temperature != 0 && !sets_temperature(machine_start_gcode, &["M140", "M190"], false) {
        output.extend_from_slice(
            format!(
                "M190 S{bed_temperature} ; set bed temperature and wait for it to be reached\n"
            )
            .as_bytes(),
        );
    }
    append_nozzle_temperatures(output, traversal, machine_start_gcode, flavor);
    bed_temperature
}

pub(super) fn overall_chamber_temperature(
    traversal: &PreparedPostClassicTraversal,
    used_extruders: &[usize],
) -> i32 {
    chamber_settings(traversal, used_extruders).1
}

pub(super) fn append_chamber_shutdown(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    used_extruders: &[usize],
) {
    let (active, temperature) = chamber_settings(traversal, used_extruders);
    if active && temperature > 0 {
        output.extend_from_slice(b"M141 S0;set chamber_temperature\n");
    }
}

pub(super) fn append_chamber_startup(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    machine_start_gcode: &str,
    used_extruders: &[usize],
) {
    let (active, temperature) = chamber_settings(traversal, used_extruders);
    if !active
        || temperature <= 0
        || sets_temperature(machine_start_gcode, &["M141", "M191"], false)
    {
        return;
    }
    let auxiliary_fan = traversal.resolved.views.runtime_gcode.auxiliary_fan.0;
    if auxiliary_fan {
        output.extend_from_slice(b"M106 P2 S255 \n");
    }
    output.extend_from_slice(
        format!("M191 S{temperature} ;set chamber_temperature and wait for it to be reached\n")
            .as_bytes(),
    );
    if auxiliary_fan {
        output.extend_from_slice(b"M106 P2 S0 \n");
    }
}

fn chamber_settings(
    traversal: &PreparedPostClassicTraversal,
    used_extruders: &[usize],
) -> (bool, i32) {
    let filament = &traversal.resolved.views.full.filament.print;
    used_extruders
        .iter()
        .copied()
        .fold((false, 0), |(active, maximum), index| {
            let enabled = filament
                .activate_chamber_temp_control
                .0
                .get(index)
                .or_else(|| filament.activate_chamber_temp_control.0.first())
                .is_some_and(|value| value.0);
            (
                active || enabled,
                maximum.max(filament_int(&filament.chamber_temperature, index)),
            )
        })
}

pub(super) fn filament_int(values: &crate::OrcaInts, index: usize) -> i32 {
    values
        .0
        .get(index)
        .or_else(|| values.0.first())
        .map_or(0, |value| value.0)
}

fn append_nozzle_temperatures(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    machine_start_gcode: &str,
    flavor: GCodeFlavor,
) {
    if sets_temperature(
        machine_start_gcode,
        &["M104", "M109"],
        flavor == GCodeFlavor::RepRapFirmware,
    ) {
        return;
    }

    let settings = &traversal.resolved.views.full;
    let runtime = &traversal.resolved.views.runtime_gcode;
    let mut used = used_filaments(traversal);
    let single_extruder_multi_material = runtime.single_extruder_multi_material.0;
    if single_extruder_multi_material {
        used.truncate(1);
    }
    let multiple_extruders =
        settings.project.print.nozzle_diameter.0.len() > 1 && !single_extruder_multi_material;
    let first = used[0];
    let ooze_prevention = settings.process.print.ooze_prevention.0;
    for tool in used {
        let mut temperature = filament_int(
            &settings.filament.print.nozzle_temperature_initial_layer,
            tool,
        );
        if ooze_prevention && tool != first {
            let idle = filament_int(&settings.filament.print.idle_temperature, tool);
            temperature = if idle == 0 {
                temperature + settings.process.print.standby_temperature_delta.0
            } else {
                idle
            };
        }
        if temperature > 0 {
            append_nozzle_temperature(
                output,
                flavor,
                temperature as u32,
                false,
                multiple_extruders.then_some(tool),
            );
        }
    }
}

fn used_filaments(traversal: &PreparedPostClassicTraversal) -> Vec<usize> {
    let mut used = extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if used.is_empty() {
        used.push(0);
    }
    used.sort_unstable();
    used.dedup();
    used
}

fn sets_temperature(gcode: &str, commands: &[&str], include_g10: bool) -> bool {
    // GCode.cpp:528-589 scans a NUL-terminated string, not whitespace tokens.
    let bytes = gcode.as_bytes();
    let at = |index: usize| bytes.get(index).copied().unwrap_or(0);
    let commands = [commands[0], commands[1]].map(|command| command[1..].parse::<i32>().unwrap());
    let mut position = 0;
    let mut temperature_set = false;
    while at(position) != 0 {
        while matches!(at(position), b' ' | b'\t') {
            position += 1;
        }
        if at(position) == b'M' || (include_g10 && at(position) == b'G') {
            let is_gcode = at(position) == b'G';
            position += 1;
            let (number, end) = strtol(bytes, position);
            let number = number as i32;
            // Preserve C++'s (conversion && is_gcode) ? G10 : M-code test.
            let matched = if end != position && is_gcode {
                number == 10
            } else {
                commands.contains(&number)
            };
            if matched {
                position = end;
                temperature_set |= !is_gcode;
                let (end, parameter_sets_temperature) =
                    scan_temperature_parameters(bytes, position);
                position = end;
                temperature_set |= parameter_sets_temperature;
            }
        }
        while !matches!(at(position), b'\r' | b'\n' | 0) {
            position += 1;
        }
        while matches!(at(position), b'\r' | b'\n') {
            position += 1;
        }
    }
    temperature_set
}

// GCode.cpp:561-583, the matched-command parameter loop.
fn scan_temperature_parameters(bytes: &[u8], mut position: usize) -> (usize, bool) {
    let at = |index: usize| bytes.get(index).copied().unwrap_or(0);
    let mut temperature_set = false;
    while !matches!(at(position), b';' | b'\r' | b'\n' | 0) {
        while matches!(at(position), b' ' | b'\t') {
            position += 1;
        }
        if at(position) == b'S' {
            position += 1;
            while matches!(at(position), b' ' | b'\t') {
                position += 1;
            }
            let (_, end) = strtol(bytes, position);
            if end > position {
                position = end;
                temperature_set = true;
            }
        } else {
            while !matches!(at(position), b' ' | b'\t' | b';' | b'\r' | b'\n' | 0) {
                position += 1;
            }
        }
    }
    (position, temperature_set)
}

fn strtol(bytes: &[u8], start: usize) -> (i64, usize) {
    let at = |index: usize| bytes.get(index).copied().unwrap_or(0);
    let mut end = start;
    while matches!(at(end), b' ' | b'\t' | b'\n' | b'\r' | 11 | 12) {
        end += 1;
    }
    let negative = at(end) == b'-';
    if matches!(at(end), b'+' | b'-') {
        end += 1;
    }
    let digits = end;
    let mut value = 0_i64;
    while at(end).is_ascii_digit() {
        let digit = i64::from(at(end) - b'0');
        value = value.saturating_mul(10);
        value = if negative {
            value.saturating_sub(digit)
        } else {
            value.saturating_add(digit)
        };
        end += 1;
    }
    if end == digits {
        (0, start)
    } else {
        (value, end)
    }
}
