use crate::{
    GCodeFlavor,
    project_slice::{extruders, perimeters::classic::traversal::PreparedPostClassicTraversal},
};

#[cfg(test)]
mod tests;

fn first_layer_bed_temperature(traversal: &PreparedPostClassicTraversal) -> i32 {
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
        crate::BedTemperatureFormula::FirstFilament => temps.first().copied().unwrap_or(0),
        crate::BedTemperatureFormula::HighestTemp => temps.iter().copied().max().unwrap_or(0),
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
) -> i32 {
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    if flavor == GCodeFlavor::Klipper {
        return 0;
    }

    let bed_temperature = first_layer_bed_temperature(traversal);
    if !sets_temperature(machine_start_gcode, &["M140", "M190"], false) {
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
                temperature,
                multiple_extruders.then_some(tool),
            );
        }
    }
}

fn append_nozzle_temperature(
    output: &mut Vec<u8>,
    flavor: GCodeFlavor,
    temperature: i32,
    tool: Option<usize>,
) {
    let (command, parameter) = match flavor {
        GCodeFlavor::RepRapFirmware => ("G10", 'S'),
        GCodeFlavor::Mach3 | GCodeFlavor::Machinekit => ("M104", 'P'),
        _ => ("M104", 'S'),
    };
    let tool = match (flavor, tool) {
        (_, None) => String::new(),
        (GCodeFlavor::RepRapFirmware, Some(tool)) => format!(" P{tool}"),
        (_, Some(tool)) => format!(" T{tool}"),
    };
    output.extend_from_slice(
        format!("{command} {parameter}{temperature}{tool} ; set nozzle temperature\n").as_bytes(),
    );
}

fn sets_temperature(gcode: &str, commands: &[&str], include_g10: bool) -> bool {
    gcode.lines().any(|line| {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        let mut words = code.split_ascii_whitespace();
        let Some(command) = words.next() else {
            return false;
        };
        commands.contains(&command)
            || (include_g10
                && command == "G10"
                && words.any(|word| {
                    word.strip_prefix('S')
                        .is_some_and(|value| value.parse::<i32>().is_ok())
                }))
    })
}
