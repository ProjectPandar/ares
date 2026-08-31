use crate::project_slice::{
    extruders,
    island_print_order::{self, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod brim;
mod cooling;
mod expression;
mod file_start;
mod finish;
pub(super) mod footprint;
mod header;
mod layer_gcode;
mod lexer;
mod machine;
pub(in crate::project_slice) mod motion;
mod object;
mod offset;
mod placeholders;
mod processor;
mod skirt;
mod small_area;
mod spiral_vase;
mod tags;
mod template;
#[cfg(test)]
mod tests;
mod timelapse;
mod value;
use crate::{GenerationMetadata, SliceError};

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
    let brim = brim::BrimPlan::generate(traversal)?;
    let skirt = skirt::SkirtPlan::generate(traversal, brim.as_ref())?;
    let first_layer_bounds =
        footprint::first_layer_bounds(traversal, skirt.as_ref(), brim.as_ref());
    let mut output = Vec::new();
    file_start::append(&mut output, traversal, metadata, first_layer_bounds)?;
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
    let (bed_cache, start_position) =
        machine::append_start(&mut output, traversal, metadata, first_layer_bounds)?;
    let options = motion::MotionOptions::from_traversal(traversal);
    let small_area_flow = small_area::from_traversal(traversal)?;
    let model_offset = footprint::model_center(traversal).unwrap_or_default();
    let model_offset = (
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.0).unwrap()),
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.1).unwrap()),
    );
    let extruder_offset = offset::initial_extruder(traversal);
    let offset = (
        model_offset.0 - extruder_offset.0,
        model_offset.1 - extruder_offset.1,
    );
    let mut state = motion::EmitState {
        offset,
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        small_area_flow,
        tags: tags::Tags::of(traversal),
        spiral_vase: traversal.resolved.views.full.process.print.spiral_mode.0,
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
    let mut spiral = spiral_vase::SpiralVaseFilter::from_traversal(traversal, brim.is_some());
    let max_layer_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum();
    let layer_change_template =
        layer_gcode::LayerChangeTemplate::new(traversal, metadata, first_layer_bounds);
    let runtime_gcode = &traversal.resolved.views.runtime_gcode;
    let traditional_timelapse = !runtime_gcode.time_lapse_gcode.0.is_empty()
        && ((runtime_gcode.printer_structure == crate::PrinterStructure::I3
            && !traversal.resolved.views.full.process.print.spiral_mode.0)
            || traversal
                .resolved
                .views
                .full
                .project
                .print
                .nozzle_diameter
                .0
                .len()
                > 1);
    // The mid-layer insert only fires on I3 printers (`GCode.cpp:5455-5461`);
    // corexy/multi-nozzle traditional prints fall through to the layer-end
    // sequence (`GCode.cpp:5527-5546`).
    let traditional_interlude =
        traditional_timelapse && runtime_gcode.printer_structure == crate::PrinterStructure::I3;
    let mut second_layer_done = false;
    let object_count = prepared.objects.len();
    for (object_index, object) in prepared.objects.iter_mut().enumerate() {
        let labels = object::ObjectLabels::from_traversal(traversal, object_index);
        let object_layer_count = object.len();
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter_mut().enumerate() {
            if layer_index == 0 {
                layer_gcode::append_print_preamble(
                    &mut output,
                    traversal,
                    metadata,
                    start_position.as_ref(),
                    first_layer_bounds,
                )?;
            }
            let layer_output_start = output.len();
            cooling.begin_layer(&mut output, layer_index);
            state.part_fan_speed = cooling.provisional_part_speed();
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
            let timelapse_context = timelapse::Context {
                traversal,
                layer: timelapse::TimelapseLayer {
                    index: layer_index,
                    z: f64::from(layer_z),
                    max_z: f64::from(layer_z),
                },
                metadata,
                first_layer_bounds,
            };
            let timelapse_at_layer_change =
                !tags.is_bbl() && !runtime_gcode.time_lapse_gcode.0.is_empty();
            layer_gcode::append_before_layer_change_gcode(
                &mut output,
                traversal,
                layer_index,
                f64::from(layer_z),
                &layer_change_template,
            )?;
            motion::flush_pending_retract_wipe(&mut output, &mut state);
            // Pending object-end labels flush after the layer-change
            // retract/wipe, before the layer-change gcode
            // (`GCode.cpp:5699` `change_layer`).
            motion::append_exclude_end(&mut output, &mut state);
            if layer_index == 0 {
                motion::retract_before_layer(&mut output, &mut state);
            }
            if timelapse_at_layer_change {
                timelapse::append_and_track(&mut output, &mut state, timelapse_context)?;
            }
            spiral.append_layer_z(&mut output, layer_index, f64::from(layer_z));
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
            state.source_layer_z = precise_layer_z;
            state.layer_index = layer_index;
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
            let spiral_body_layer = spiral.is_body_layer(layer, layer_index, f64::from(layer_z));
            state.spiral_vase_layer = spiral_body_layer;
            if let Some(labels) = &labels {
                labels.queue_start(&mut output, &mut state, emit_labels);
            }
            let timelapse_inserted =
                motion::emit_layer(&mut output, layer, geometry, &mut state, |output, state| {
                    timelapse::append_traditional(
                        traditional_interlude,
                        output,
                        state,
                        timelapse_context,
                    )
                })?;
            if let Some(labels) = &labels {
                labels.queue_stop(&mut output, &mut state, emit_labels, timelapse_inserted);
            } else if timelapse_inserted {
                motion::defer_layer_retraction(&mut state);
            } else {
                motion::end_layer_for_timelapse(&mut output, &mut state);
            }
            append_layer_end_timelapse(
                &mut output,
                &mut state,
                timelapse_inserted,
                timelapse_at_layer_change,
                traditional_timelapse,
                timelapse_context,
            )?;
            spiral.process_layer(
                &mut output,
                spiral_vase::Layer {
                    start: layer_output_start,
                    enabled: spiral_body_layer,
                    final_layer: object_index + 1 == object_count
                        && layer_index + 1 == object_layer_count,
                    z: f64::from(layer_z),
                    height: f64::from(layer_height),
                },
            );
            cooling.finish_layer(&mut output, layer_output_start);
        }
    }
    let emitted_layer_count = header::finalize_layer_count(&mut output, tags);
    // The final compatible layer has no following layer marker to flush its
    // deferred retraction. Flush only retract/wipe (not a travel lift) before
    // end G-code (`GCode.cpp` final object teardown).
    motion::retract_for_print_end(&mut output, &mut state);
    motion::append_exclude_end(&mut output, &mut state);
    finish::append(
        &mut output,
        traversal,
        max_layer_z,
        metadata,
        first_layer_bounds,
    )?;
    machine::append_completion_controls(&mut output, traversal);
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    let used_filament = finish::account_used_filament(&output);
    let (total_weight, total_cost) =
        finish::append_filament_stats(&mut output, traversal, used_filament);
    finish::append_compatible_stats(
        &mut output,
        traversal,
        total_weight,
        total_cost,
        emitted_layer_count,
    );
    output.push(b'\n');
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
        used_filament,
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
#[expect(
    clippy::too_many_arguments,
    reason = "the layer-end timelapse branch mirrors the Orca insertion flags"
)]
fn append_layer_end_timelapse(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    inserted: bool,
    at_layer_change: bool,
    traditional: bool,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    if inserted || at_layer_change {
        return Ok(());
    }
    if traditional {
        // `GCode.cpp:5538-5546` `add_object_change_labels`: pending
        // object-end labels flush before the layer-end template.
        motion::append_exclude_end(output, state);
    }
    timelapse::append_and_track(output, state, context)
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
