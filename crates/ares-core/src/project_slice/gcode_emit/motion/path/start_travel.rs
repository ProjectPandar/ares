use super::{PathProperties, retraction, travel_emit};
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, LiftMode, append_object_start, arc, begin_path_travel, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
    travel,
};

pub(super) struct Request<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
}

pub(super) fn emit(output: &mut Vec<u8>, state: &mut EmitState, request: Request<'_>) {
    let Request {
        first_scaled,
        first_x,
        first_y,
        properties,
        geometry,
    } = request;
    let first_position = !state.positioned;
    let layer_change_travel = state.layer_change_travel_pending && !first_position;
    let slope_start_z = properties
        .slope
        .filter(|slope| (slope.z_begin - slope.z_end).abs() > super::SOURCE_EPSILON_MM)
        .map(|slope| {
            state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * slope.z_begin
        });
    let slope_needs_z_travel = slope_start_z.is_some_and(|target| {
        (state.scarf_z.unwrap_or(state.layer_z) - target).abs() >= super::SOURCE_EPSILON_MM
    });
    let target_z = slope_start_z.unwrap_or(state.layer_z);
    let needs_travel =
        first_position || state.last_scaled_position != Some(first_scaled) || slope_needs_z_travel;
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let mut travel_set_layer_z = false;
    let mut eager_lifted_travel = false;
    let mut unclear_position_travel = false;
    if needs_travel {
        begin_path_travel(output, state, properties.feature, travel_distance);
        let inside_internal_surface = travel::inside_internal_surfaces(
            geometry.internal_surfaces,
            arc::Point {
                x: state.x,
                y: state.y,
            },
            arc::Point {
                x: first_x,
                y: first_y,
            },
            geometry.scale,
            state.offset,
        );
        let skip_retraction = super::can_skip_retraction(
            state.options.reduce_infill_retraction,
            state.options.has_sparse_infill,
            state.last_feature,
            properties.is_perimeter,
            inside_internal_surface,
        );
        // Orca decides the travel retraction purely on distance, role, and
        // overhang crossing (`GCode.cpp:7359`, `needs_retraction`); the very
        // first travel of the print retracts the same way when it is long
        // enough (the `; printing object` label precedes it and the queued
        // `M486 S<n>` flushes after the retraction, `GCode.cpp:7467`).
        let retract = !state.retracted
            && travel_distance >= state.options.retraction_minimum_travel
            && !skip_retraction;
        if retract {
            travel::retract_and_lift(output, state);
        }
        // TODO(port): Orca arms the boundary on every layer
        // (`GCode.cpp:5345-5347`) and routes the object's first travel with
        // the external-only planner (`use_external_mp_once`, direct when the
        // destination lies inside the external contour). The rectangle
        // router below does not model that yet, so keep the first layer
        // direct until `AvoidCrossingPerimeters` is ported.
        let mut route = if state.options.reduce_crossing_wall
            && state.layer_index > 0
            && !matches!(properties.feature, "Skirt" | "Brim")
        {
            super::avoid_crossing::route(super::avoid_crossing::Request {
                start: arc::Point {
                    x: state.x,
                    y: state.y,
                },
                end: arc::Point {
                    x: first_x,
                    y: first_y,
                },
                geometry,
                offset: state.offset,
                inset: state.options.crossing_boundary_inset,
                after_skirt: state.last_feature == Some("Skirt"),
            })
        } else {
            Vec::new()
        };
        route.push(arc::Point {
            x: first_x,
            y: first_y,
        });
        route.dedup();
        let first_travel = route[0];
        let (travel_x, travel_y) = (first_travel.x, first_travel.y);
        append_object_start(output, state);
        // `GCodeWriter::travel_to_xyz` (`GCodeWriter.cpp:685-707`) only raises
        // the travel destination for a hop scheduled with this travel
        // (`m_to_lift`); a nozzle already lifted by an earlier sequence (the
        // layer-end timelapse retract, a deferred layer-change lift) travels
        // flat in XY and the unlift descends at the destination.
        let lifted_for_travel = state.pending_lift.is_some();
        // The print's first travel has no known source position, so Orca
        // cannot slope/spiral from it (`GCodeWriter.cpp:travel_to_xyz`
        // skips those branches when `is_current_position_clear()` is
        // false). A pending normal lift still raises before the XY move
        // (`NormalLift` `slop_move`), and both paths end with the separate
        // `_travel_to_z` re-statement from the unclear-position branch.
        let first_travel_lift = if first_position {
            state.pending_lift.take()
        } else {
            travel::emit_pending_lift(
                output,
                arc::Point {
                    x: travel_x,
                    y: travel_y,
                },
                state,
            );
            None
        };
        if state.spiral_vase
            && state.lifted
            && !first_position
            && (layer_change_travel
                || (state.source_layer_z + state.options.z_hop) - state.options.z_hop
                    > state.source_layer_z)
        {
            travel_emit::xyz(
                output,
                travel_x,
                travel_y,
                state.layer_z,
                state.travel_feedrate,
            );
            travel_set_layer_z = true;
        } else if state.template_lifted && state.lifted && !first_position {
            travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            state.template_lifted = false;
        } else if state.lifted {
            eager_lifted_travel = true;
            if !lifted_for_travel {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            } else if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz(
                    output,
                    travel_x,
                    travel_y,
                    state.layer_z + state.options.z_hop,
                    state.travel_feedrate,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            }
        } else if layer_change_travel && state.retracted {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            } else if state.lifted {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            } else {
                travel_emit::xyz(output, travel_x, travel_y, target_z, state.travel_feedrate);
                travel_set_layer_z = true;
            }
        } else if state.retracted
            && first_position
            && state.options.z_hop > 0.0
            && travel::lift_is_allowed(state)
        {
            let mode = first_travel_lift.unwrap_or_else(|| travel::lift_mode_for(state, true));
            if mode == LiftMode::Normal {
                output.extend_from_slice(
                    format!(
                        "G1 Z{} F{}\n",
                        format_z(state.layer_z + state.options.z_hop),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                state.current_feedrate = state.travel_feedrate;
                travel_emit::xy_without_feed(output, travel_x, travel_y);
            } else {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            }
            output.extend_from_slice(
                format!("G1 Z{}\n", format_z(state.layer_z + state.options.z_hop)).as_bytes(),
            );
            state.lifted = true;
        } else if layer_change_travel {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
                output.extend_from_slice(format!("G1 Z{}\n", format_z(target_z)).as_bytes());
            } else {
                travel_emit::xyz(output, travel_x, travel_y, target_z, state.travel_feedrate);
            }
            travel_set_layer_z = true;
        } else if let Some(z) = slope_start_z {
            travel_emit::xyz(output, travel_x, travel_y, z, state.travel_feedrate);
            travel_set_layer_z = true;
        } else {
            travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            // The print's first travel starts from an unknown position, so
            // Orca cannot fold Z into the XY move and re-states it as a
            // separate descend (`GCodeWriter.cpp:travel_to_xyz` unclear-
            // position branch emits XY then `_travel_to_z`).
            if first_position {
                output.extend_from_slice(format!("G1 Z{}\n", format_z(target_z)).as_bytes());
                unclear_position_travel = true;
            }
        }
        state.x = travel_x;
        state.y = travel_y;
        for point in &route[1..] {
            travel_emit::xy_without_feed(output, point.x, point.y);
            state.x = point.x;
            state.y = point.y;
        }
        state.last_scaled_position = Some(first_scaled);
        state.positioned = true;
        state.current_feedrate = state.travel_feedrate;
    } else if layer_change_travel {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(target_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        travel_set_layer_z = true;
        state.current_feedrate = state.travel_feedrate;
    }
    if let Some(z) = slope_start_z {
        state.scarf_z = Some(z);
    }
    state.layer_change_travel_pending = false;
    append_object_start(output, state);
    // Orca `_extrude` (`GCode.cpp:6378-6385`): the print's first extrusion
    // re-states Z to sync the writer with the planned layer height
    // (`_last_pos_undefined`). It applies whenever the travel emitted the
    // Z as a separate unclear-position descend — the eager-lift branch or
    // the unknown-source first travel. The lazy-lift branch's descend above
    // already models it.
    if (eager_lifted_travel || unclear_position_travel) && first_position && !travel_set_layer_z {
        output.extend_from_slice(format!("G1 Z{}\n", format_z(state.layer_z)).as_bytes());
    }
    if state.retracted {
        if first_position
            && state.options.z_hop > 0.0
            && !state.lifted
            && travel::lift_is_allowed(state)
        {
            output.extend_from_slice(
                format!("G1 Z{}\n", format_z(state.layer_z + state.options.z_hop)).as_bytes(),
            );
            state.lifted = true;
        }
        if state.lifted && !travel_set_layer_z {
            let z = slope_start_z.unwrap_or(state.layer_z);
            output.extend_from_slice(format!("G1 Z{}\n", format_z(z)).as_bytes());
            state.scarf_z = slope_start_z;
        }
        let retraction_length = state.options.retraction_length;
        let unretract = extrusion::coordinate(state, retraction_length);
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(unretract),
                format_axis(state.options.deretraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.deretraction_feedrate;
        state.retracted = false;
        state.lifted = false;
        state.template_lifted = false;
    }
}
