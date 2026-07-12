use crate::gcode_machine_start_runtime_placeholders as runtime_placeholders;
use crate::gcode_placeholders::MachineStartPlaceholderContext;
use crate::{PrintPathRole, SliceError, SliceOptions};

pub(crate) fn machine_start_gcode(
    options: &SliceOptions,
    adaptive_bed_mesh: Option<&crate::gcode_adaptive_bed_mesh::AdaptiveBedMeshPlaceholders>,
    first_layer_print: Option<
        &crate::gcode_first_layer_print_placeholders::FirstLayerPrintPlaceholders,
    >,
    context: MachineStartPlaceholderContext,
) -> Result<String, SliceError> {
    let template = options.machine_start_gcode()?;
    if template.is_empty() {
        return Ok(String::new());
    }
    let values = options.auxiliary_fan_placeholders()?;
    let exhaust_fan_speed_num =
        format_placeholder_integers(&options.during_print_exhaust_fan_speed_num_values()?);
    let flush_placeholders = options.flush_placeholders()?;
    let flush_volumetric_speeds =
        format_placeholder_numbers(flush_placeholders.flush_volumetric_speeds());
    let flush_temperatures = format_placeholder_integers(flush_placeholders.flush_temperatures());
    let filament_cooling_before_tower = options.filament_cooling_before_tower_placeholder()?;
    let min_vitrification_temperature = options.temperature_vitrification()?.value().to_string();
    let chamber_temperature = format_placeholder_integers(&options.chamber_temperature_values()?);
    let overall_chamber_temperature = options.overall_chamber_temperature()?.to_string();
    let total_layer_count = context.total_layer_count.to_string();
    let num_extruders = context.num_extruders.to_string();
    let max_print_z = context.max_print_z.to_string();
    let initial_extruder = "0";
    let filament_change = options.filament_change_options()?;
    let has_single_extruder_multi_material_priming =
        has_single_extruder_multi_material_priming_placeholder(
            filament_change.single_extruder_multi_material_priming(),
        );
    let current_hotend = if template.contains("[current_hotend]") {
        Some(current_hotend_placeholder(options)?)
    } else {
        None
    };
    let first_tools = first_tools_placeholder(context.num_extruders);
    let print_bed_placeholders =
        if crate::gcode_print_bed_placeholders::template_contains_placeholder(template) {
            Some(crate::gcode_print_bed_placeholders::placeholders(options)?)
        } else {
            None
        };
    let print_bed_min = print_bed_placeholders
        .as_ref()
        .map(|placeholders| placeholders.min_list())
        .unwrap_or_default();
    let print_bed_max = print_bed_placeholders
        .as_ref()
        .map(|placeholders| placeholders.max_list())
        .unwrap_or_default();
    let print_bed_size = print_bed_placeholders
        .as_ref()
        .map(|placeholders| placeholders.size_list())
        .unwrap_or_default();
    let first_layer_print_min = first_layer_print
        .map(|placeholders| placeholders.min_list())
        .unwrap_or_default();
    let first_layer_print_max = first_layer_print
        .map(|placeholders| placeholders.max_list())
        .unwrap_or_default();
    let first_layer_print_size = first_layer_print
        .map(|placeholders| placeholders.size_list())
        .unwrap_or_default();
    let first_layer_center_no_wipe_tower = first_layer_print
        .map(|placeholders| placeholders.center_list())
        .unwrap_or_default();
    let outer_wall_volumetric_speed =
        format_placeholder_number(outer_wall_volumetric_speed(options)?);
    let retract_length = format_placeholder_number(options.start_gcode_retract_length()?);
    let retraction_distance_when_cut =
        format_placeholder_number(options.retraction_distance_when_cut()?);
    let retraction_distance_when_ec = if template.contains("[retraction_distance_when_ec]") {
        Some(format_placeholder_number(
            options.retraction_distance_when_ec()?,
        ))
    } else {
        None
    };
    let retraction_distances_when_cut =
        format_placeholder_numbers(&options.retraction_distances_when_cut()?);
    let retraction_distances_when_ec =
        format_placeholder_nullable_numbers(&options.retraction_distances_when_ec()?);
    let long_retractions_when_cut = format_placeholder_bools(&options.long_retractions_when_cut()?);
    let long_retractions_when_ec =
        format_placeholder_nullable_bools(&options.long_retractions_when_ec()?);
    let long_retraction_when_cut = if options.long_retraction_when_cut()? {
        "1"
    } else {
        "0"
    };
    let long_retraction_when_ec = if options.long_retraction_when_ec()? {
        "1"
    } else {
        "0"
    };
    let z_offset = format_placeholder_number(options.z_offset()?);
    let first_layer_height = format_placeholder_number(first_layer_height_placeholder(options)?);
    let max_print_height = max_print_height_placeholder(options)?.to_string();
    let first_layer_temperature = options.first_layer_nozzle_temperature_values()?[0].to_string();
    let temperature = options.machine_start_temperature_placeholder()?.to_string();
    let first_layer_bed_temperature_values = options.first_layer_bed_temperature_values()?;
    let bed_temperature_initial_layer =
        format_placeholder_integers(&first_layer_bed_temperature_values);
    let first_layer_bed_temperature = first_layer_bed_temperature_values[0].to_string();
    let bed_temperature_initial_layer_single =
        options.first_layer_bed_temperature()?.value().to_string();
    let bed_temperature =
        format_placeholder_integers(&options.other_layer_bed_temperature_values()?);
    let is_all_bbl_filament = if options.is_all_bbl_filament()? {
        "1"
    } else {
        "0"
    };
    let has_tpu_in_first_layer = if options.has_tpu_in_first_layer()? {
        "1"
    } else {
        "0"
    };
    let rendered = template
        .replace(
            "[during_print_exhaust_fan_speed_num]",
            &exhaust_fan_speed_num,
        )
        .replace(
            "[max_additional_fan]",
            &format_placeholder_number(values.max_additional_fan()),
        )
        .replace(
            "[first_x_layer_fan_speed]",
            &format_placeholder_number(values.first_x_layer_fan_speed()),
        )
        .replace(
            "[close_additional_fan_first_x_layers]",
            &values.close_additional_fan_first_x_layers().to_string(),
        )
        .replace(
            "[additional_fan_full_speed_layer]",
            &values.additional_fan_full_speed_layer().to_string(),
        )
        .replace("[flush_volumetric_speeds]", &flush_volumetric_speeds)
        .replace("[flush_temperatures]", &flush_temperatures)
        .replace(
            "[filament_cooling_before_tower]",
            &filament_cooling_before_tower,
        )
        .replace(
            "[min_vitrification_temperature]",
            &min_vitrification_temperature,
        )
        .replace("[chamber_temperature]", &chamber_temperature)
        .replace(
            "[overall_chamber_temperature]",
            &overall_chamber_temperature,
        )
        .replace("[total_layer_count]", &total_layer_count)
        .replace("[num_extruders]", &num_extruders)
        .replace("[initial_tool]", initial_extruder)
        .replace("[initial_extruder]", initial_extruder)
        .replace("[current_extruder]", initial_extruder)
        .replace(
            "[current_hotend]",
            current_hotend.as_deref().unwrap_or_default(),
        )
        .replace("[current_object_idx]", "0")
        .replace("[first_tools]", &first_tools)
        .replace("[first_filaments]", &first_tools)
        .replace("[first_non_support_tools]", &first_tools)
        .replace("[first_non_support_filaments]", &first_tools)
        .replace("[initial_no_support_tool]", initial_extruder)
        .replace("[initial_no_support_extruder]", initial_extruder)
        .replace("[initial_no_support_hotend]", initial_extruder)
        .replace("[has_wipe_tower]", "0")
        .replace(
            "[has_single_extruder_multi_material_priming]",
            has_single_extruder_multi_material_priming,
        )
        .replace("[total_toolchanges]", "0")
        .replace("[print_bed_min]", &print_bed_min)
        .replace("[print_bed_max]", &print_bed_max)
        .replace("[print_bed_size]", &print_bed_size)
        .replace("[first_layer_print_min]", first_layer_print_min)
        .replace("[first_layer_print_max]", first_layer_print_max)
        .replace("[first_layer_print_size]", first_layer_print_size)
        .replace(
            "[first_layer_center_no_wipe_tower]",
            first_layer_center_no_wipe_tower,
        )
        .replace(
            "[outer_wall_volumetric_speed]",
            &outer_wall_volumetric_speed,
        )
        .replace("[retract_length]", &retract_length)
        .replace(
            "[retraction_distance_when_cut]",
            &retraction_distance_when_cut,
        )
        .replace(
            "[retraction_distance_when_ec]",
            retraction_distance_when_ec.as_deref().unwrap_or_default(),
        )
        .replace(
            "[retraction_distances_when_cut]",
            &retraction_distances_when_cut,
        )
        .replace(
            "[retraction_distances_when_ec]",
            &retraction_distances_when_ec,
        )
        .replace("[long_retractions_when_cut]", &long_retractions_when_cut)
        .replace("[long_retractions_when_ec]", &long_retractions_when_ec)
        .replace("[long_retraction_when_cut]", long_retraction_when_cut)
        .replace("[long_retraction_when_ec]", long_retraction_when_ec)
        .replace("[z_offset]", &z_offset)
        .replace("[first_layer_height]", &first_layer_height)
        .replace("[max_print_height]", &max_print_height)
        .replace("[max_print_z]", &max_print_z)
        .replace("[temperature]", &temperature)
        .replace("[first_layer_temperature]", &first_layer_temperature)
        .replace(
            "[bed_temperature_initial_layer]",
            &bed_temperature_initial_layer,
        )
        .replace("[bed_temperature_initial_layer_vector]", "")
        .replace("[bed_temperature]", &bed_temperature)
        .replace("[bbl_bed_temperature_gcode]", "0")
        .replace("[is_all_bbl_filament]", is_all_bbl_filament)
        .replace("[has_tpu_in_first_layer]", has_tpu_in_first_layer)
        .replace(
            "[first_layer_bed_temperature]",
            &first_layer_bed_temperature,
        )
        .replace(
            "[bed_temperature_initial_layer_single]",
            &bed_temperature_initial_layer_single,
        );
    let rendered = crate::gcode_machine_start_stat_placeholders::render(rendered);
    let rendered =
        runtime_placeholders::render(rendered, options, context.filament_count, first_layer_print)?;
    let rendered = if let Some(mesh) = adaptive_bed_mesh {
        rendered
            .replace("[adaptive_bed_mesh_min]", &mesh.min_list())
            .replace("[adaptive_bed_mesh_min_0]", mesh.min_x())
            .replace("[adaptive_bed_mesh_min_1]", mesh.min_y())
            .replace("[adaptive_bed_mesh_max]", &mesh.max_list())
            .replace("[adaptive_bed_mesh_max_0]", mesh.max_x())
            .replace("[adaptive_bed_mesh_max_1]", mesh.max_y())
            .replace("[bed_mesh_probe_count]", &mesh.probe_count_list())
            .replace(
                "[bed_mesh_probe_count_0]",
                &mesh.probe_count_x().to_string(),
            )
            .replace(
                "[bed_mesh_probe_count_1]",
                &mesh.probe_count_y().to_string(),
            )
            .replace("[bed_mesh_algo]", mesh.algorithm())
    } else {
        rendered
    };
    Ok(crate::gcode_format::ensure_trailing_newline(rendered))
}

fn has_single_extruder_multi_material_priming_placeholder(
    single_extruder_multi_material_priming: bool,
) -> &'static str {
    let _ = single_extruder_multi_material_priming;
    "0"
}

fn format_placeholder_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn first_layer_height_placeholder(options: &SliceOptions) -> Result<f64, SliceError> {
    options.initial_layer_print_height()
}

fn max_print_height_placeholder(options: &SliceOptions) -> Result<i32, SliceError> {
    let value = match options.values().get("printable_height") {
        Some(serde_json::Value::Number(number)) => number.as_f64(),
        Some(serde_json::Value::String(text)) => text.parse().ok(),
        Some(_) => None,
        None => crate::options::registry::option_definition("printable_height")
            .and_then(|definition| definition.default_value.parse().ok()),
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("printable_height must be a finite number".to_owned())
    })?;

    if value.is_finite() && value >= 0.0 {
        Ok((value + 0.5).floor() as i32)
    } else {
        Err(SliceError::InvalidInput(
            "printable_height must be non-negative".to_owned(),
        ))
    }
}

fn current_hotend_placeholder(options: &SliceOptions) -> Result<String, SliceError> {
    let hotend = match options.values().get("printer_model") {
        Some(serde_json::Value::String(model)) if model == "Bambu Lab X2D" => "-1",
        Some(serde_json::Value::String(_)) | None => "0",
        Some(_) => {
            return Err(SliceError::InvalidInput(
                "printer_model must be a string".to_owned(),
            ));
        }
    };
    Ok(hotend.to_owned())
}

fn outer_wall_volumetric_speed(options: &SliceOptions) -> Result<f64, SliceError> {
    let extrusion_options = options.extrusion_options()?;
    let speed_options = options.speed_options()?;
    let layer_height = options.layer_height()?;
    let extrusion_per_mm =
        extrusion_options.extrusion_per_mm(PrintPathRole::ExternalPerimeter, layer_height)?;
    let filament_area = std::f64::consts::PI * (speed_options.filament_diameter_mm() / 2.0).powi(2);
    let material_mm3_per_mm = extrusion_per_mm * filament_area;
    let uncapped = speed_options.external_perimeter_speed_mm_s() * material_mm3_per_mm;
    Ok(uncapped.min(speed_options.filament_max_volumetric_speed_mm3_s()))
}

fn format_placeholder_numbers(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format_placeholder_number(*value))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_placeholder_nullable_numbers(values: &[Option<f64>]) -> String {
    values
        .iter()
        .map(|value| match value {
            Some(value) => format_placeholder_number(*value),
            None => "nil".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_placeholder_integers(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn first_tools_placeholder(num_extruders: usize) -> String {
    (0..num_extruders)
        .map(|index| if index == 0 { "0" } else { "-1" })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_placeholder_bools(values: &[bool]) -> String {
    values
        .iter()
        .map(|value| if *value { "1" } else { "0" })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_placeholder_nullable_bools(values: &[Option<bool>]) -> String {
    values
        .iter()
        .map(|value| match value {
            Some(true) => "1",
            Some(false) => "0",
            None => "nil",
        })
        .collect::<Vec<_>>()
        .join(",")
}
