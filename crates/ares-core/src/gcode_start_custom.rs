use crate::gcode_placeholders::MachineStartPlaceholderContext;
use crate::gcode_writer::GCodeWriter;
use crate::options::{
    ChamberTemperatureControl, ExhaustFanControl, FirstLayerBedTemperature, GCodeFlavor,
};
use crate::{LayerPrintPaths, SliceError, SliceOptions};

const INITIAL_FILAMENT_EXTRUDER_ID: usize = 0;

pub(crate) struct StartGCodeCommand<'a> {
    pub(crate) writer: &'a GCodeWriter,
    pub(crate) options: &'a SliceOptions,
    pub(crate) gcode_flavor: GCodeFlavor,
    pub(crate) first_layer_bed_temperature: FirstLayerBedTemperature,
    pub(crate) chamber_temperature_control: ChamberTemperatureControl,
    pub(crate) exhaust_fan_control: ExhaustFanControl,
    pub(crate) layer_print_paths: &'a [LayerPrintPaths],
    pub(crate) total_layer_count: usize,
    pub(crate) num_extruders: usize,
    pub(crate) filament_count: usize,
    pub(crate) max_print_z: i32,
}

pub(crate) fn start_gcode(command: StartGCodeCommand<'_>) -> Result<String, SliceError> {
    let machine_start_gcode = crate::gcode_adaptive_bed_mesh::machine_start_gcode(
        command.options,
        command.gcode_flavor,
        command.layer_print_paths,
        MachineStartPlaceholderContext {
            total_layer_count: command.total_layer_count,
            num_extruders: command.num_extruders,
            filament_count: command.filament_count,
            max_print_z: command.max_print_z,
        },
    )?;
    let mut gcode = crate::gcode_startup::first_layer_bed_temperature_command(
        command.writer,
        command.gcode_flavor,
        command.first_layer_bed_temperature,
        &machine_start_gcode,
    );
    gcode.push_str(
        &crate::gcode_startup::first_layer_nozzle_temperature_commands(
            command.writer,
            command.gcode_flavor,
            command.options,
            command.num_extruders,
            &machine_start_gcode,
        )?,
    );
    gcode.push_str(&crate::gcode_startup::chamber_temperature_startup_command(
        command.writer,
        command.gcode_flavor,
        command.chamber_temperature_control,
        &machine_start_gcode,
    ));
    gcode.push_str(&crate::gcode_startup::exhaust_fan_startup_command(
        command.writer,
        command.gcode_flavor,
        command.exhaust_fan_control,
    ));
    gcode.push_str(&machine_start_gcode);
    gcode.push_str(&crate::gcode_placeholders::filament_start_gcode(
        command.options,
        INITIAL_FILAMENT_EXTRUDER_ID,
    )?);
    Ok(gcode)
}
