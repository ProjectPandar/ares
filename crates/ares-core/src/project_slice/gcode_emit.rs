use crate::project_slice::{
    island_print_order::{self, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod expression;
mod finish;
mod footprint;
mod header;
mod lexer;
mod motion;
mod object;
mod template;
#[cfg(test)]
mod tests;
mod value;
use crate::{GenerationMetadata, SliceError};

pub(super) fn emit(
    prepared: &PreparedPostIslandPrintOrder,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let mut output = Vec::new();
    header::append_header(&mut output, metadata, &prepared.objects, traversal);
    if let Some(config) = &traversal.config_block {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    append_machine_limits(&mut output, traversal);
    append_machine_start(&mut output, traversal)?;
    let options = motion::MotionOptions::from_traversal(traversal);
    let mut state = motion::EmitState {
        offset: footprint::model_center(traversal).unwrap_or_default(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        ..Default::default()
    };
    let labels = object::ObjectLabels::from_traversal(traversal);
    for (object_index, object) in prepared.objects.iter().enumerate() {
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter().enumerate() {
            if layer_index == 0 {
                append_print_preamble(&mut output);
            }
            output.extend_from_slice(b"; CHANGE_LAYER\n");
            let record_layer_height = traversal
                .objects
                .first()
                .and_then(|object| object.records.get(layer_index))
                .and_then(|record| record.as_ref())
                .map_or(0.0, |record| record.layer_height);
            precise_layer_z += record_layer_height;
            let layer_z = precise_layer_z as f32;
            let layer_height = layer_z - previous_layer_z;
            previous_layer_z = layer_z;
            output.extend_from_slice(
                format!(
                    "; Z_HEIGHT: {}\n; LAYER_HEIGHT: {}\n",
                    format_processor_float(f64::from(layer_z)),
                    format_processor_float(f64::from(layer_height))
                )
                .as_bytes(),
            );
            if layer_index == 0 {
                output.extend_from_slice(b"G1 E-.4 F1800\n");
                state.retracted = true;
            }
            append_layer_change(&mut output, traversal, layer_index, f64::from(layer_z))?;
            motion::begin_layer(
                &mut output,
                &mut state,
                layer_index,
                f64::from(layer_z),
                f64::from(layer_height),
            );
            if let Some(labels) = &labels {
                labels.append_printing(&mut output);
                motion::begin_object_travel(&mut output, &mut state);
                labels.append_start_label(&mut output);
            }
            motion::emit_layer(
                &mut output,
                layer,
                motion::LayerGeometry {
                    internal_surfaces: island_print_order::internal_surfaces(
                        prepared,
                        object_index,
                        layer_index,
                    ),
                    scale: traversal.scale,
                },
                &mut state,
            )?;
            if let Some(labels) = &labels {
                labels.append_stopping(&mut output);
                labels.append_stop_label(&mut output);
            }
        }
    }

    fn append_print_preamble(output: &mut Vec<u8>) {
        output.extend_from_slice(
            b"; filament start gcode\n;VT0\nG90\nG21\nM83 ; use relative distances for extrusion\n",
        );
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\nM106 S0\nM106 P2 S0\n");
    }

    fn append_layer_change(
        output: &mut Vec<u8>,
        traversal: &PreparedPostClassicTraversal,
        layer_index: usize,
        layer_z: f64,
    ) -> Result<(), SliceError> {
        let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
        if !template.is_empty() {
            let mut config =
                value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
            config.insert("current_extruder", value::Value::number(0.0));
            config.insert("layer_num", value::Value::number(layer_index as f64));
            config.insert("layer_z", value::Value::number(layer_z));
            config.insert("overall_chamber_temperature", value::Value::number(0.0));
            if let Some(value) = config
                .get("temperature_vitrification")
                .and_then(|value| value.index(0))
                .cloned()
            {
                config.insert("min_vitrification_temperature", value);
            }
            if let Some(value) = config
                .get("fan_max_speed")
                .and_then(|value| value.index(0))
                .cloned()
            {
                config.insert("max_additional_fan", value);
            }
            let rendered = template::render(template, &config).map_err(|error| {
                SliceError::InvalidInput(format!(
                    "invalid project layer-change G-code template: {error}"
                ))
            })?;
            output.extend_from_slice(rendered.as_bytes());
            output.push(b'\n');
        }
        output.extend_from_slice(b";_SET_FAN_SPEED_CHANGING_LAYER\n");
        Ok(())
    }
    let max_layer_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum();
    finish::append(&mut output, traversal, max_layer_z)?;
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    Ok(output)
}
fn format_processor_float(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let precision = (5 - magnitude).max(0) as usize;
    let mut formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }
    formatted
}

fn append_machine_limits(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let travel_acceleration = if traversal.resolved.views.full.printer.gcode.gcode_flavor
        == crate::GCodeFlavor::MarlinLegacy
    {
        machine.machine_max_acceleration_extruding.clone()
    } else {
        machine.machine_max_acceleration_travel.clone()
    };
    output.extend_from_slice(b"M73 P0 R0\n");
    output.extend_from_slice(
        format!(
            "M201 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_acceleration_x),
            first(&machine.machine_max_acceleration_y),
            first(&machine.machine_max_acceleration_z),
            first(&machine.machine_max_acceleration_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M203 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_speed_x),
            first(&machine.machine_max_speed_y),
            first(&machine.machine_max_speed_z),
            first(&machine.machine_max_speed_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M204 P{} R{} T{}\n",
            first(&machine.machine_max_acceleration_extruding),
            first(&machine.machine_max_acceleration_retracting),
            first(&travel_acceleration),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M205 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/sec\n",
            first(&machine.machine_max_jerk_x),
            first(&machine.machine_max_jerk_y),
            first(&machine.machine_max_jerk_z),
            first(&machine.machine_max_jerk_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(b"M106 S0\nM106 P2 S0\n");
}

fn first(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

fn append_machine_start(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
) -> Result<(), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    if template.is_empty() {
        return Ok(());
    }
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    config.insert("current_extruder", value::Value::number(0.0));
    config.insert("current_hotend", value::Value::number(-1.0));
    config.insert("next_extruder", value::Value::number(0.0));
    config.insert("next_hotend", value::Value::number(-1.0));
    config.insert("initial_no_support_extruder", value::Value::number(0.0));
    config.insert("initial_no_support_hotend", value::Value::number(-1.0));
    config.insert("overall_chamber_temperature", value::Value::number(0.0));
    config.insert(
        "hold_chamber_temp_for_flat_print",
        value::Value::Bool(false),
    );
    let max_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum::<f64>();
    config.insert("max_print_z", value::Value::number(max_z));
    config.insert("max_layer_z", value::Value::number(max_z));
    config.insert(
        "total_layer_count",
        value::Value::number(
            traversal
                .objects
                .first()
                .map_or(0, |object| object.records.len()) as f64,
        ),
    );
    config.insert("layer_num", value::Value::number(0.0));
    if let Some((min_x, min_y, size_x, size_y)) = footprint::first_layer_bounds(traversal) {
        config.insert(
            "first_layer_print_min",
            value::Value::List(vec![
                value::Value::number(min_x),
                value::Value::number(min_y),
            ]),
        );
        config.insert(
            "first_layer_print_size",
            value::Value::List(vec![
                value::Value::number(size_x),
                value::Value::number(size_y),
            ]),
        );
    }
    let filament_count = traversal.resolved.logical_filament_count;
    config.insert(
        "first_non_support_filaments",
        value::Value::List(
            (0..filament_count)
                .map(|_| value::Value::number(-1.0))
                .collect(),
        ),
    );
    config.insert(
        "first_filaments",
        value::Value::List(
            (0..filament_count)
                .map(|index| value::Value::number(index as f64 - 1.0))
                .collect(),
        ),
    );
    for (target, source) in [
        ("flush_temperatures", "nozzle_temperature_range_high"),
        ("flush_volumetric_speeds", "filament_max_volumetric_speed"),
        (
            "first_layer_temperature",
            "nozzle_temperature_initial_layer",
        ),
    ] {
        if let Some(value) = config.get(source).cloned() {
            config.insert(target, value);
        }
    }
    if let Some(value) = config
        .get("hot_plate_temp_initial_layer")
        .and_then(|value| value.index(0))
        .cloned()
    {
        config.insert("bed_temperature_initial_layer_single", value);
    }
    output.extend_from_slice(b"; FEATURE: Custom\n");
    let rendered = template::render(template, &config).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project G-code template: {error}"))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    if !rendered.ends_with('\n') {
        output.push(b'\n');
    }
    Ok(())
}
