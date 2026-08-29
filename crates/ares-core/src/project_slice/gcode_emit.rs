use crate::project_slice::{
    extruders,
    island_print_order::{self, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod brim;
mod cooling;
mod expression;
mod finish;
pub(super) mod footprint;
mod header;
mod layer_gcode;
mod lexer;
mod machine;
mod motion;
mod object;
mod placeholders;
mod processor;
mod skirt;
mod tags;
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
    let tags = tags::Tags::of(traversal);
    header::append_header(&mut output, metadata, &prepared.objects, traversal);
    // `GCode.cpp` + Orca export layout: BBL keeps the config block up front;
    // the compatible flavor moves it after the tail statistics.
    if tags.is_bbl()
        && let Some(config) = &traversal.config_block
    {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    object::append_definitions(&mut output, traversal);
    machine::append_first_line_m73(&mut output);
    // The Marlin-family machine envelope prints before the start G-code
    // (`GCode.cpp:2819`), followed by the start G-code (`GCode.cpp:3137`).
    machine::append_limits(&mut output, traversal);
    let (bed_cache, start_position) = machine::append_start(&mut output, traversal, metadata)?;
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
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        tags: tags::Tags::of(traversal),
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
    let layer_change_template = layer_gcode::LayerChangeTemplate::new(traversal, metadata);
    let mut second_layer_done = false;
    let skirt = skirt::SkirtPlan::generate(traversal)?;
    let brim = brim::BrimPlan::generate(traversal)?;
    for (object_index, object) in prepared.objects.iter_mut().enumerate() {
        let labels = object::ObjectLabels::from_traversal(traversal, object_index);
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter_mut().enumerate() {
            let layer_output_start = output.len();
            if layer_index == 0 {
                layer_gcode::append_print_preamble(
                    &mut output,
                    traversal,
                    metadata,
                    start_position.as_ref(),
                )?;
            }
            cooling.begin_layer(&mut output, layer_index);
            state.part_fan_speed = cooling.part_speed();
            state.physical_fan_speed = state.part_fan_speed;
            let tags = state.tags;
            output.extend_from_slice(tags.layer_change().as_bytes());
            output.push(b'\n');
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
            let header = format!(
                "{}\n{}\n",
                tags.z(&format_processor_float(f64::from(layer_z))),
                tags.height(&format_processor_float(f64::from(layer_height))),
            );
            output.extend_from_slice(header.as_bytes());
            layer_gcode::append_before_layer_change_gcode(
                &mut output,
                traversal,
                layer_index,
                f64::from(layer_z),
                metadata,
            )?;
            motion::flush_pending_retract_wipe(&mut output, &mut state);
            // Pending object-end labels flush after the layer-change
            // retract/wipe, before the layer-change gcode
            // (`GCode.cpp:5699` `change_layer`).
            motion::append_exclude_end(&mut output, &mut state);
            if layer_index == 0 {
                motion::retract_before_layer(&mut output, &mut state);
            }
            layer_gcode::append_layer_change(
                &mut output,
                traversal,
                layer_index,
                f64::from(layer_z),
                &layer_change_template,
            )?;
            // A deferred previous-layer retraction lifts above the new layer's
            // print Z (`GCodeWriter::travel_to_z` during layer transition).
            state.layer_z = f64::from(layer_z);
            motion::flush_pending_retract_lift(&mut output, &mut state);
            motion::begin_layer(
                &mut output,
                &mut state,
                layer_index,
                f64::from(layer_z),
                f64::from(layer_height),
            );
            // Second-layer transition: bed temperature for the remaining
            // layers (`GCode.cpp:4777-4830`), once per slice.
            if layer_index == 1 && !second_layer_done {
                second_layer_done = true;
                machine::append_second_layer_transition(&mut output, traversal, bed_cache);
            }
            let lower_boundary_lines = traversal.objects[object_index]
                .lower_slices(layer_index)
                .into_iter()
                .flatten()
                .flat_map(crate::geometry::ExPolygon::lines)
                .collect::<Vec<_>>();
            let lower_boundary = (!lower_boundary_lines.is_empty())
                .then(|| crate::geometry::LineDistanceTree::new(&lower_boundary_lines));
            let geometry = motion::LayerGeometry {
                internal_surfaces: island_print_order::internal_surfaces(
                    &prepared.predecessor,
                    object_index,
                    layer_index,
                ),
                scale: traversal.scale,
                previous_layer_boundary: lower_boundary.as_ref(),
            };
            // The skirt prints once per layer before any object content
            // (`GCode.cpp:4388+`), on the layers it covers.
            if object_index == 0
                && let Some(plan) = &skirt
            {
                plan.emit(
                    &mut output,
                    skirt::SkirtLayer {
                        index: layer_index,
                        height_mm: f64::from(layer_height),
                    },
                    geometry,
                    &mut state,
                );
            }
            if layer_index == 0
                && object_index == 0
                && let Some(plan) = &brim
            {
                plan.emit(&mut output, geometry, &mut state);
            }
            if let Some(labels) = &labels {
                labels.queue_start(&mut output, &mut state, emit_labels);
            }
            motion::emit_layer(&mut output, layer, geometry, &mut state)?;
            if let Some(labels) = &labels {
                labels.queue_stop(&mut output, &mut state, emit_labels);
            } else {
                motion::end_layer_for_timelapse(&mut output, &mut state);
            }
            timelapse::append(
                &mut output,
                traversal,
                timelapse::TimelapseLayer {
                    index: layer_index,
                    z: f64::from(layer_z),
                    max_z: max_layer_z,
                },
                metadata,
            )?;
            cooling.finish_layer(&mut output, layer_output_start);
        }
    }
    // The final compatible layer has no following layer marker to flush its
    // deferred retraction. Flush only retract/wipe (not a travel lift) before
    // end G-code (`GCode.cpp` final object teardown).
    motion::flush_pending_retract_wipe(&mut output, &mut state);
    motion::append_exclude_end(&mut output, &mut state);
    finish::append(&mut output, traversal, max_layer_z, metadata)?;
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    let used_filament = finish::account_used_filament(&output);
    finish::append_filament_stats(&mut output, traversal, used_filament);
    if !tags.is_bbl() {
        // Compatible-flavor tail statistics: layer count, klipper-style
        // time placeholders, then the config block.
        let layers = prepared
            .objects
            .first()
            .map(|object| object.len())
            .unwrap_or(0);
        output.extend_from_slice(
            format!(
                "; total layers count = {layers}\n\
; estimated printing time (normal mode) = 0s\n\
; estimated first layer printing time (normal mode) = 0s\n\n"
            )
            .as_bytes(),
        );
        if let Some(config) = &traversal.config_block {
            output.extend_from_slice(config);
        }
    }
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
            gcode_flavor: traversal.resolved.views.full.printer.gcode.gcode_flavor,
            bbl_printer: state.tags.is_bbl(),
        },
    ))
}
pub(super) fn format_processor_float(value: f64) -> String {
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
