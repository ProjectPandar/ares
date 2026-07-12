use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MachineStartPlaceholderContext {
    pub(crate) total_layer_count: usize,
    pub(crate) num_extruders: usize,
    pub(crate) filament_count: usize,
    pub(crate) max_print_z: i32,
}

pub(crate) fn file_start_gcode(options: &SliceOptions) -> Result<String, SliceError> {
    let template = options.file_start_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let rendered = template
        .replace(
            "{print_time_sec}",
            crate::gcode_reserved_tags::PRINT_TIME_SEC,
        )
        .replace(
            "{used_filament_length}",
            crate::gcode_reserved_tags::USED_FILAMENT_LENGTH,
        );
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn before_layer_change_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
    max_layer_z: &str,
) -> Result<String, SliceError> {
    let template = options.before_layer_change_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = layer_num.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", layer_z)
        .replace("[layer_z]", layer_z)
        .replace("{max_layer_z}", max_layer_z)
        .replace("[max_layer_z]", max_layer_z);
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn layer_change_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
    max_layer_z: &str,
) -> Result<String, SliceError> {
    let template = options.layer_change_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = layer_num.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", layer_z)
        .replace("[layer_z]", layer_z)
        .replace("{max_layer_z}", max_layer_z)
        .replace("[max_layer_z]", max_layer_z);
    let physical_extruder_id = options.physical_extruder_id_for_logical(0)?.to_string();
    let rendered = replace_placeholder(
        rendered,
        "most_used_physical_extruder_id",
        &physical_extruder_id,
    );
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn time_lapse_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
    max_layer_z: &str,
) -> Result<String, SliceError> {
    let template = options.time_lapse_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = layer_num.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", layer_z)
        .replace("[layer_z]", layer_z)
        .replace("{max_layer_z}", max_layer_z)
        .replace("[max_layer_z]", max_layer_z);
    let physical_extruder_id = options.physical_extruder_id_for_logical(0)?.to_string();
    let rendered = replace_placeholder(
        rendered,
        "most_used_physical_extruder_id",
        &physical_extruder_id,
    );
    let rendered =
        replace_placeholder(rendered, "curr_physical_extruder_id", &physical_extruder_id);
    Ok(ensure_trailing_newline(rendered))
}

fn replace_placeholder(template: String, key: &str, value: &str) -> String {
    template
        .replace(&format!("{{{key}}}"), value)
        .replace(&format!("[{key}]"), value)
}

#[derive(Clone, Copy)]
pub(crate) struct ChangeExtrusionRoleGCodeCommand<'a> {
    pub(crate) layer_num: usize,
    pub(crate) layer_z: &'a str,
    pub(crate) extrusion_role: &'a str,
    pub(crate) last_extrusion_role: &'a str,
}

pub(crate) fn change_extrusion_role_gcode(
    options: &SliceOptions,
    command: ChangeExtrusionRoleGCodeCommand<'_>,
) -> Result<String, SliceError> {
    role_change_gcode(options.change_extrusion_role_gcode()?, command)
}

pub(crate) fn process_change_extrusion_role_gcode(
    options: &SliceOptions,
    command: ChangeExtrusionRoleGCodeCommand<'_>,
) -> Result<String, SliceError> {
    role_change_gcode(options.process_change_extrusion_role_gcode()?, command)
}

pub(crate) fn filament_change_extrusion_role_gcode(
    options: &SliceOptions,
    command: ChangeExtrusionRoleGCodeCommand<'_>,
) -> Result<String, SliceError> {
    role_change_gcode(options.filament_change_extrusion_role_gcode()?, command)
}

fn role_change_gcode(
    template: &str,
    command: ChangeExtrusionRoleGCodeCommand<'_>,
) -> Result<String, SliceError> {
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = command.layer_num.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", command.layer_z)
        .replace("[layer_z]", command.layer_z)
        .replace("{extrusion_role}", command.extrusion_role)
        .replace("[extrusion_role]", command.extrusion_role)
        .replace("{last_extrusion_role}", command.last_extrusion_role)
        .replace("[last_extrusion_role]", command.last_extrusion_role);
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn machine_end_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
    max_layer_z: &str,
    filament_extruder_id: usize,
) -> Result<String, SliceError> {
    let template = options.machine_end_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = layer_num.to_string();
    let filament_extruder_id = filament_extruder_id.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", layer_z)
        .replace("[layer_z]", layer_z)
        .replace("{max_layer_z}", max_layer_z)
        .replace("[max_layer_z]", max_layer_z)
        .replace("{filament_extruder_id}", &filament_extruder_id)
        .replace("[filament_extruder_id]", &filament_extruder_id);
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn filament_end_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
    max_layer_z: &str,
    filament_extruder_id: usize,
) -> Result<String, SliceError> {
    let template = options.filament_end_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let layer_num = layer_num.to_string();
    let filament_extruder_id = filament_extruder_id.to_string();
    let rendered = template
        .replace("{layer_num}", &layer_num)
        .replace("[layer_num]", &layer_num)
        .replace("{layer_z}", layer_z)
        .replace("[layer_z]", layer_z)
        .replace("{max_layer_z}", max_layer_z)
        .replace("[max_layer_z]", max_layer_z)
        .replace("{filament_extruder_id}", &filament_extruder_id)
        .replace("[filament_extruder_id]", &filament_extruder_id);
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn filament_start_gcode(
    options: &SliceOptions,
    filament_extruder_id: usize,
) -> Result<String, SliceError> {
    let template = options.filament_start_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let filament_extruder_id = filament_extruder_id.to_string();
    let rendered = template
        .replace("{filament_extruder_id}", &filament_extruder_id)
        .replace("[filament_extruder_id]", &filament_extruder_id);
    Ok(ensure_trailing_newline(rendered))
}

pub(crate) fn machine_start_gcode(
    options: &SliceOptions,
    adaptive_bed_mesh: Option<&crate::gcode_adaptive_bed_mesh::AdaptiveBedMeshPlaceholders>,
    first_layer_print: Option<
        &crate::gcode_first_layer_print_placeholders::FirstLayerPrintPlaceholders,
    >,
    context: MachineStartPlaceholderContext,
) -> Result<String, SliceError> {
    crate::gcode_machine_start_placeholders::machine_start_gcode(
        options,
        adaptive_bed_mesh,
        first_layer_print,
        context,
    )
}

fn ensure_trailing_newline(mut text: String) -> String {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}
