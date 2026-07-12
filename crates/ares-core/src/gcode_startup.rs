// Source: OrcaSlicer/src/libslic3r/GCode.cpp

use crate::gcode_writer::GCodeWriter;
use crate::options::{
    ChamberTemperatureControl, ExhaustFanControl, FirstLayerBedTemperature, GCodeFlavor,
};
use crate::{SliceError, SliceOptions};

pub(crate) fn first_layer_bed_temperature_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    temperature: FirstLayerBedTemperature,
    machine_start_gcode: &str,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper
        || contains_command(machine_start_gcode, &["M140", "M190"])
    {
        return String::new();
    }
    writer.set_bed_temperature(temperature.value(), true)
}

pub(crate) fn first_layer_nozzle_temperature_commands(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    options: &SliceOptions,
    num_extruders: usize,
    machine_start_gcode: &str,
) -> Result<String, SliceError> {
    options.validate_startup_nozzle_temperature_options()?;
    if gcode_flavor == GCodeFlavor::Klipper {
        return Ok(String::new());
    }
    let commands = if gcode_flavor == GCodeFlavor::RepRapFirmware {
        &["M104", "M109", "G10"][..]
    } else {
        &["M104", "M109"][..]
    };
    if contains_command(machine_start_gcode, commands) {
        return Ok(String::new());
    }

    if num_extruders <= 1 {
        let temperature = options.first_layer_nozzle_temperature()?;
        return Ok(if temperature.emits_command() {
            writer.set_nozzle_temperature(temperature.value(), false, None)
        } else {
            String::new()
        });
    }

    let ooze_prevention = options.ooze_prevention()?;
    let standby_delta = options.standby_temperature_delta()?;
    let mut gcode = String::new();
    for tool_index in 0..num_extruders {
        let first_layer = options
            .first_layer_nozzle_temperature_for_tool(tool_index)?
            .value();
        let temperature = if tool_index == 0 || !ooze_prevention {
            i64::from(first_layer)
        } else {
            let idle = options.idle_temperature_for_tool(tool_index)?;
            if idle > 0 {
                i64::from(idle)
            } else {
                i64::from(first_layer) + i64::from(standby_delta)
            }
        };
        if temperature > 0 {
            let temperature = u32::try_from(temperature).map_err(|_| {
                SliceError::InvalidInput(
                    "startup nozzle temperature must fit in an unsigned 32-bit integer".into(),
                )
            })?;
            gcode.push_str(&writer.set_nozzle_temperature(
                temperature,
                false,
                Some(tool_index as u32),
            ));
        }
    }
    Ok(gcode)
}

pub(crate) fn chamber_temperature_startup_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    control: ChamberTemperatureControl,
    machine_start_gcode: &str,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper
        || contains_command(machine_start_gcode, &["M141", "M191"])
    {
        return String::new();
    }
    control
        .temperature()
        .map(|temperature| writer.set_chamber_temperature(temperature, true))
        .unwrap_or_default()
}

pub(crate) fn exhaust_fan_startup_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    control: ExhaustFanControl,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper {
        return String::new();
    }
    control
        .during_print_speed()
        .map(|speed| writer.set_exhaust_fan(speed))
        .unwrap_or_default()
}

fn contains_command(gcode: &str, commands: &[&str]) -> bool {
    gcode.lines().any(|line| {
        let line = line.trim_start();
        commands.iter().any(|command| {
            line == *command
                || line
                    .strip_prefix(command)
                    .is_some_and(|tail| tail.chars().next().is_some_and(char::is_whitespace))
        })
    })
}

pub(crate) fn exhaust_fan_completion_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    control: ExhaustFanControl,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper {
        return String::new();
    }
    control
        .completion_speed()
        .map(|speed| writer.set_exhaust_fan(speed))
        .unwrap_or_default()
}
