use crate::{
    HardwareOptions, SliceError, SliceOptions, SlicingPipeline,
    gcode_writer::GCodeWriter,
    options::{
        ChamberTemperatureControl, ExhaustFanControl, FirstLayerBedTemperature, GCodeFlavor,
    },
};

pub(crate) struct FileStartCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) pipeline: &'a SlicingPipeline,
    pub(crate) options: &'a SliceOptions,
    pub(crate) gcode_flavor: GCodeFlavor,
    pub(crate) first_layer_bed_temperature: FirstLayerBedTemperature,
    pub(crate) chamber_temperature_control: ChamberTemperatureControl,
    pub(crate) exhaust_fan_control: ExhaustFanControl,
    pub(crate) hardware_options: &'a HardwareOptions,
    pub(crate) layer_height: f64,
    pub(crate) initial_layer_height: f64,
    pub(crate) machine_limits: crate::options::MachineLimits,
    pub(crate) input_shaping: crate::options::InputShapingConfig,
    pub(crate) max_print_z: i32,
}

pub(crate) fn file_start(command: FileStartCommand<'_>) -> Result<String, SliceError> {
    let mut gcode = crate::gcode_placeholders::file_start_gcode(command.options)?;
    if !crate::gcode_header::should_skip_header_for_btt_thumbnail(command.options) {
        gcode.push_str(&crate::gcode_header::format_header(
            command.pipeline,
            command.options,
            crate::gcode_header::HeaderConfig {
                layer_height: command.layer_height,
                initial_layer_height: command.initial_layer_height,
                hardware_options: command.hardware_options,
            },
        )?);
    }
    gcode.push_str(&command.writer.preamble());
    gcode.push_str(&crate::gcode_machine_limits::format_machine_envelope(
        command.gcode_flavor,
        command.machine_limits,
    ));
    gcode.push_str(&crate::gcode_input_shaping::format_input_shaping(
        command.gcode_flavor,
        command.machine_limits.emit_to_gcode,
        command.input_shaping,
    ));
    gcode.push_str(&crate::gcode_start_custom::start_gcode(
        crate::gcode_start_custom::StartGCodeCommand {
            writer: command.writer,
            options: command.options,
            gcode_flavor: command.gcode_flavor,
            first_layer_bed_temperature: command.first_layer_bed_temperature,
            chamber_temperature_control: command.chamber_temperature_control,
            exhaust_fan_control: command.exhaust_fan_control,
            layer_print_paths: command.pipeline.layer_print_paths(),
            total_layer_count: command.pipeline.layers().len(),
            num_extruders: command.hardware_options.nozzle_diameters().len(),
            filament_count: command.hardware_options.filament_diameters().len(),
            max_print_z: command.max_print_z,
        },
    )?);
    Ok(gcode)
}
