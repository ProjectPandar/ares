    while phase_index < phases.len() {
        let merged: &[(f64, schedule::LayerToPrint)] = phases[phase_index].merged;
        let phase = &phases[phase_index];
        state.layer_object_labels = None;
        let tool_ordering: &super::tool_ordering::ToolOrdering = phases[phase_index].tool_ordering;
        if by_object && group_start == 0 {
            state.most_used_extruder = tool_ordering.most_used_extruder(traversal);
        }
        if group_start >= merged.len() {
            phase_index += 1;
            group_start = 0;
            status_layer = 0;
            if let Some(phase) = phases.get(phase_index) {
                // `GCode.cpp:3359-3361`: after the finished object's layers
                // (its stop label already written) and before the instance
                // transition, disable power loss recovery once the
                // second-layer block ran with `Enable`; `:3367` then
                // resets the flag for the incoming instance (whose own
                // second layer re-arms it).
                output.extend_from_slice(
                    crate::gcode_power_loss_recovery::finish_command(
                        state.tags.is_bbl(),
                        state.options.gcode_flavor,
                        state.options.gcode_comments,
                        state.power_loss_recovery,
                    )
                    .as_bytes(),
                );
                state.power_loss_recovery = Default::default();
                // `GCode.cpp:3309-3342`: the instance transition between
                // the sequential objects; `:3369-3372` resets the
                // second-layer flag for the incoming instance, and the
                // layer height restarts from the object's own first
                // layer (the per-object `process_layers` z tracking).
                let origin = instance_origin(
                    &instances[phase.object][phase
                        .copy
                        .min(instances[phase.object].len().saturating_sub(1))..],
                    traversal.scale.factor(),
                );
                let _primed = transition::emit(
                    output,
                    state,
                    transition::Transition {
                        traversal,
                        metadata,
                        first_layer_bounds,
                        finished_objects: phase_index,
                        initial_extruder: phase.tool_ordering.first_extruder.unwrap_or(0),
                        printing_extruders: phase.tool_ordering.all_extruders().to_vec(),
                        lift_z: max_layer_z + state.options.z_hop,
                        origin,
                        bed_cache: &mut bed_cache,
                    },
                )?;
                let _ = _primed;
                cooling.reset_fans();
                previous_layer_z = 0.0;
                second_layer_done = false;
            }
            continue;
        }
        let group_z = merged[group_start].0;
        let mut group_end = group_start;
        while group_end < merged.len() && merged[group_end].0 == group_z {
            group_end += 1;
        }
        let mut entries: Vec<schedule::LayerToPrint> = merged[group_start..group_end]
            .iter()
            .map(|&(_, entry)| entry)
            .collect();
        let (cooling_layer_index, cooling_buffer_flush) = cooling_result::metadata(
            &entries,
            traversal,
            support_layers,
            group_end == merged.len(),
        );
        group_start = group_end;
        // `GCode.cpp:5149,5248`: one status per layer to print; by object, each instance restarts at 1.
        status_layer += 1;
        slicer.set_status(80, format_args!("Generating G-code: layer {status_layer}"), 0);
        if tool_ordering.tools_for_layer(group_z).is_empty() {
            cooling.empty_layer(output, cooling_layer_index, group_end == merged.len());
            continue;
        }
        // GCode.cpp:4559-4594 selects the first object layer in source order,
        // or the first support layer when no object layer exists, before dispatch.
        let layer_object = entries
            .iter()
            .find(|entry| entry.object_layer.is_some())
            .unwrap_or(&entries[0])
            .object_index;
        state.apply_object_config(&traversal.print_object_config(layer_object).0.object);
        entries.sort_by_key(|entry| print_position[entry.object_index]);
        max_layer_z = max_layer_z.max(f64::from(group_z as f32));
        // Chunk-wide inputs for the external motion planner
        // (`get_perimeter_spacing_external` averages the perimeter
        // spacing over every object with slices at this print z,
        // `AvoidCrossingPerimeters.cpp:511-529`).
        let (chunk_slices, chunk_perimeter_spacing) = group::geometry(traversal, &entries);
        // `get_boundary_external` translates every appended hole by its
        // print instance's shift (`AvoidCrossingPerimeters.cpp:1172-1177`),
        // so the external boundary's input is plate-frame while the
        // brim/skirt/tower copy above stays object-local.
        entry_builder.boundary_slices =
            group::boundary_slices(traversal, &entries, &instances);
        // The preamble fires on the first PHYSICAL layer chunk — raft layer
        // 0 included (`first_layer` is a sentinel there, so the check keys
        // off `first_group`, not `first_layer == 0`). Upstream emits the
        // filament-start g-code at the raft base, not the object base.
        if first_group {
            fan_layers_start =
                preamble::emit(output, state, traversal, metadata, first_layer_bounds)?;
            tower::emit(
                output,
                state,
                tower::Event::Initialize,
                tower::Input {
                    context: super::timelapse::Context {
                        traversal,
                        tool_ordering,
                        metadata,
                        first_layer_bounds,
                        layer: super::timelapse::TimelapseLayer {
                            index: 0,
                            z: 0.0,
                            max_z: 0.0,
                        },
                    },
                    entry: &entries[0],
                    builder: &mut entry_builder,
                    support: entries[0]
                        .support_layer
                        .map(|index| &support_layers[entries[0].object_index][index]),
                    chunk_slices: &chunk_slices,
                    spacing: chunk_perimeter_spacing,
                },
            )?;
            first_group = false;
        }
        // The cooling rewrite window opens after the preamble (the start
        // g-code is not part of the layer's buffered extrusions).
        let layer_output_start = output.len();
        cooling.begin_layer(output, cooling_layer_index);
        state.part_fan_speed = cooling.provisional_part_speed();
        // The change-layer block uses the merged chunk's leading entry
        // (`change_layer` runs once per layer chunk, `GCode.cpp:5685`).
        let slicing = traversal.objects[entries[0].object_index].slicing_parameters();
        let object_layer_over_raft = entries[0].object_layer == Some(0)
            && slicing.base_raft_layers + slicing.interface_raft_layers > 0;
        let layer_z = group_z;
        // The HEIGHT header keeps the f32-difference quirk (upstream
        // `;HEIGHT:0.200001` artifacts) while layer_z itself stays the
        // f64 print_z for template comparisons.
        let layer_height = f64::from(layer_z as f32) - f64::from(previous_layer_z as f32);
        // `GCode.cpp:4589`: `first_layer` is per-object — the leading
        // object/support/raft layer sitting on the plate.
        let first_layer = group::on_first_layer(traversal, &entries, support_layers, &raft);
        let layer_tools = tool_ordering.for_layer(group_z);
        let wiping = &layer_tools.wiping;
        farthest::begin_layer(
            state,
            traversal,
            farthest::LayerViews {
                entries: &entries,
                object_targets: farthest_object_targets,
                support: support_layers,
                raft: &raft_layers,
                instances: &instances,
            },
            layer_tools,
        );
        let mut by_tool = tool_buckets::build(tool_buckets::Input {
            traversal,
            entries: &entries,
            instances: &instances,
            copy_filter: phase.copy_filter,
            tools: tool_ordering.tools_for_layer(group_z),
            wiping,
            objects,
            support: support_layers,
            raft: &mut raft_layers,
        });
        // GCode.cpp:5139-5144 unions the actual filament instance lists.
        // Build membership before the existing tracked layer-start insertion.
        if state.tags.is_bbl() {
            state.layer_object_labels = label_inventory.layer(
                by_tool
                    .iter()
                    .flat_map(|(_, tour)| tour.instances.iter().map(|entry| entry.instance.label_id)),
            );
        }
        let boundary = boundary::append(
            output,
            state,
            &mut spiral,
            boundary::Boundary {
                traversal,
                tool_ordering,
                layer_change_template: &layer_change_template,
                metadata,
                first_layer_bounds,
            },
            boundary::LayerBoundary {
                layer_index: chunk_index,
                object_layer_over_raft,
                layer_z,
                layer_height,
                first_layer,
                second_layer_done: &mut second_layer_done,
                bed_cache: &mut bed_cache,
                custom_gcode: tool_ordering.for_layer(group_z).custom_gcode.as_ref(),
            },
        )?;
        let timelapse_context = boundary.timelapse_context;
        state.layer_index = cooling_layer_index;
        state.on_first_layer = first_layer;
        // The skirt prints once per layer before any object content
        // (`GCode.cpp:4388+`), gated by the `skirt_done` vector:
        // `Skirt::make_skirt_loops_per_extruder_1st_layer` emits when the
        // vector is empty and records the print z
        // (`GCode.cpp:4292-4295`); `..._other_layers` keeps emitting while
        // `skirt_done.size() < skirt_height` (`GCode.cpp:4311-4315`). The
        // vector is shared across the sequential by-object instances, so a
        // later object restarting at the same z does not re-emit. Raft
        // chunks emit the skirt around the raft polygons (the skirt layer
        // index is the raft layer index — with a raft, the covered layers
        // start at the raft flange).
        // `GCode.cpp:4306-4328`: `make_skirt_extrusion` also requires the
        // NEW print z to advance past the last recorded one
        // (`skirt_done.back() < print_z - EPSILON`) — a later object
        // restarting at a lower z neither re-emits nor consumes budget.
        // The per-group accounting drives emission: `m_skirt_group_done[group_idx]`
        // (`GCode.cpp:5240-5250`, cleared once per export at :2036) is
        // threaded through `generate_skirt`, whose `make_skirt_extrusion`
        // gates on has_skirt, the skirt_height budget and print-z advance
        // (`GCode.cpp:4306-4330`) — SkirtSchedule::next is that port, so
        // the chunk position never gates the skirt.
        skirt_emit::emit(
            output,
            state,
            skirt_emit::SkirtEmission {
                timelapse_context,
                entries: &entries,
                support: support_layers,
                skirt,
                raft: &raft,
                print_z: layer_z,
                schedule: &mut skirt_schedule,
                entry_builder: &mut entry_builder,
                chunk_slices: &chunk_slices,
                chunk_perimeter_spacing,
            },
        )?;
        // `GCode.cpp:5281-5284, 5302-5305`: the wrapping-detection retract +
        // template insert fire once per layer inside the per-extruder loop
        // — AFTER the combined skirt (`GCode.cpp:5244-5252`) but BEFORE the
        // toolchange and every instance block (object skirt group, brim,
        // `; printing object` marker, content).
        let has_tower = layer_tools.has_wipe_tower && state.wipe_tower.is_some();
        if has_tower {
            state.wipe_tower.as_mut().unwrap().next_layer();
        }
        let last_tool = by_tool.last().map(|(tool, _)| *tool);
        let mut final_entry = entries[0];
        for (tool, tool_entries) in &mut by_tool {
            wrapping::emit(output, state, &timelapse_context)?;
            if let Some(instances) = &filament_instances {
                state.filament_instances_code = Some(
                    instances.encode(tool_entries.instances.iter().map(|entry| entry.instance.label_id)),
                );
            }
            let needs_change = !state.extruder_selected || state.extruders.active_id() != *tool;
            if has_tower {
                tower::emit(
                    output,
                    state,
                    tower::Event::Change {
                        tool: *tool,
                        finish_layer: Some(*tool) == last_tool,
                    },
                    tower::Input {
                        context: timelapse_context,
                        entry: &entries[0],
                        builder: &mut entry_builder,
                        support: entries[0]
                            .support_layer
                            .map(|index| &support_layers[entries[0].object_index][index]),
                        chunk_slices: &chunk_slices,
                        spacing: chunk_perimeter_spacing,
                    },
                )?;
                state.last_feature = Some("Prime tower");
            } else if needs_change {
                super::tool_transition::emit(output, state, *tool, timelapse_context)?;
            }
            if needs_change {
                last_object_copy = None;
            }
            let entry_count = tool_entries.visits.len();
            for (pass, entry_position) in [tool_buckets::Pass::Wiping, tool_buckets::Pass::Normal]
                .into_iter()
                .filter(|pass| *pass == tool_buckets::Pass::Normal || wiping.has_overrides())
                .flat_map(|pass| (0..entry_count).map(move |position| (pass, position)))
            {
                let visit = &tool_entries.visits[entry_position];
                final_entry = tool_entries.instances[visit.instance_idx].source;
                tool_entry::emit(
                    output,
                    state,
                    &mut tool_entries.instances[visit.instance_idx],
                    tool_entry::Input {
                        visit,
                        traversal,
                        pass,
                        purging_finished: pass == tool_buckets::Pass::Normal
                            && entry_position == 0
                            && wiping.has_overrides(),
                        support_layers,
                        raft: &raft,
                        labels: &labels,
                        emit_labels,
                        traditional_interlude,
                        traditional_timelapse: timelapse_gate.traditional_timelapse,
                        timelapse_context,
                        is_group_end: pass == tool_buckets::Pass::Normal
                            && entry_position + 1 == entry_count
                            && Some(*tool) == last_tool,
                        last_object_copy: &mut last_object_copy,
                        entry_builder: &mut entry_builder,
                        chunk_slices: &chunk_slices,
                        chunk_perimeter_spacing,
                        brim,
                        skirt,
                        object_skirt_done: &mut object_skirt_done,
                        spiral: &mut spiral,
                        layer_z,
                        layer_height,
                        object_layer_counts: &object_layer_counts,
                        layer_output_start,
                    },
                )?;
            }
        }
        // An empty final visit can bypass the per-entry tail.
        layer_end::append_farthest_fallback(output, state, timelapse_context)?;
        cooling.finish_layer(output, layer_output_start, cooling_buffer_flush);
        if state.wipe_tower.is_some() {
            final_chunk = Some((tool_ordering, final_entry, chunk_slices, chunk_perimeter_spacing));
        }
        previous_layer_z = layer_z;
        chunk_index += 1;
        state.adaptive_pa_layer = chunk_index;
    }
