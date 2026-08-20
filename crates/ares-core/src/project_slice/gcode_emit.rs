use crate::project_slice::{
    island_print_order::{self, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod cooling;
mod expression;
mod finish;
pub(super) mod footprint;
mod header;
mod lexer;
mod machine;
mod motion;
mod object;
mod processor;
mod template;
#[cfg(test)]
mod tests;
mod timelapse;
mod value;
use crate::{GenerationMetadata, SliceError};

pub(in crate::project_slice) use motion::simplify_points;

pub(super) fn emit(
    prepared: &mut PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let mut output = Vec::new();
    header::append_header(&mut output, metadata, &prepared.objects, traversal);
    if let Some(config) = &traversal.config_block {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    machine::append_limits(&mut output, traversal);
    machine::append_start(&mut output, traversal)?;
    let options = motion::MotionOptions::from_traversal(traversal);
    let offset = footprint::model_center(traversal).unwrap_or_default();
    let offset = (
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(offset.0).unwrap()),
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(offset.1).unwrap()),
    );
    let mut state = motion::EmitState {
        offset,
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        ..Default::default()
    };
    let emit_labels = traversal
        .resolved
        .views
        .full
        .process
        .print
        .gcode_label_objects
        .0;
    let mut cooling = cooling::CoolingState::from_traversal(traversal);
    let max_layer_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum();
    for (object_index, object) in prepared.objects.iter_mut().enumerate() {
        let labels = emit_labels
            .then(|| object::ObjectLabels::from_traversal(traversal, object_index))
            .flatten();
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter_mut().enumerate() {
            let layer_output_start = output.len();
            if layer_index == 0 {
                append_print_preamble(&mut output);
            }
            cooling.begin_layer(&mut output, layer_index);
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
            let lower_boundary_lines = traversal.objects[object_index]
                .lower_slices(layer_index)
                .into_iter()
                .flatten()
                .flat_map(crate::geometry::ExPolygon::lines)
                .collect::<Vec<_>>();
            let lower_boundary = (!lower_boundary_lines.is_empty())
                .then(|| crate::geometry::LineDistanceTree::new(&lower_boundary_lines));
            motion::emit_layer(
                &mut output,
                layer,
                motion::LayerGeometry {
                    internal_surfaces: island_print_order::internal_surfaces(
                        &prepared.predecessor,
                        object_index,
                        layer_index,
                    ),
                    scale: traversal.scale,
                    previous_layer_boundary: lower_boundary.as_ref(),
                },
                &mut state,
            )?;
            if let Some(labels) = &labels {
                labels.append_stopping(&mut output);
            }
            motion::end_layer_for_timelapse(&mut output, &mut state);
            if let Some(labels) = &labels {
                labels.append_stop_label(&mut output);
            }
            timelapse::append(
                &mut output,
                traversal,
                layer_index,
                f64::from(layer_z),
                max_layer_z,
            )?;
            cooling.finish_layer(&mut output, layer_output_start);
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
    finish::append(&mut output, traversal, max_layer_z)?;
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    finish::append_filament_stats(&mut output, traversal, state.filament_used);
    Ok(processor::process(
        output,
        !traversal.resolved.views.full.printer.gcode.disable_m73.0,
        traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .machine_load_filament_time
            .0,
        processor::ProcessorLimits {
            print_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_extruding,
            ),
            retract_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_retracting,
            ),
            travel_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_travel,
            ),
        },
    ))
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
