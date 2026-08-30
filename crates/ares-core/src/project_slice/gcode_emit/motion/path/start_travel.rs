use super::{PathProperties, retraction, travel_emit};
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, append_object_start, arc, begin_path_travel, extrusion,
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
    let needs_travel = first_position || state.last_scaled_position != Some(first_scaled);
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let mut travel_set_layer_z = false;
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
        let retract = !first_position
            && !state.retracted
            && travel_distance >= state.options.retraction_minimum_travel
            && !skip_retraction;
        if retract {
            travel::retract_and_lift(output, state);
        }
        append_object_start(output, state);
        travel::emit_pending_lift(
            output,
            arc::Point {
                x: first_x,
                y: first_y,
            },
            state,
        );
        if state.spiral_vase
            && state.lifted
            && !first_position
            && (layer_change_travel
                || (state.source_layer_z + state.options.z_hop) - state.options.z_hop
                    > state.source_layer_z)
        {
            travel_emit::xyz(
                output,
                first_x,
                first_y,
                state.layer_z,
                state.travel_feedrate,
            );
            travel_set_layer_z = true;
        } else if state.template_lifted && state.lifted && !first_position {
            travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            state.template_lifted = false;
        } else if state.lifted && !first_position {
            if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z + state.options.z_hop,
                    state.travel_feedrate,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(first_x),
                        format_axis(first_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            }
        } else if layer_change_travel && state.retracted {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            } else if state.lifted {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(first_x),
                        format_axis(first_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            } else {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z,
                    state.travel_feedrate,
                );
                travel_set_layer_z = true;
            }
        } else if state.retracted
            && first_position
            && state.options.z_hop > 0.0
            && travel::lift_is_allowed(state)
        {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 Z{} F{}\n",
                        format_z(state.layer_z + state.options.z_hop),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                output.extend_from_slice(
                    format!("G1 X{} Y{}\n", format_axis(first_x), format_axis(first_y)).as_bytes(),
                );
                state.lifted = true;
            }
        } else if layer_change_travel {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
                output.extend_from_slice(format!("G1 Z{}\n", format_z(state.layer_z)).as_bytes());
            } else {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z,
                    state.travel_feedrate,
                );
            }
            travel_set_layer_z = true;
        } else {
            travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
        }
        state.x = first_x;
        state.y = first_y;
        state.last_scaled_position = Some(first_scaled);
        state.positioned = true;
        state.current_feedrate = state.travel_feedrate;
    } else if layer_change_travel {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        travel_set_layer_z = true;
        state.current_feedrate = state.travel_feedrate;
    }
    state.layer_change_travel_pending = false;
    append_object_start(output, state);
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
            output.extend_from_slice(format!("G1 Z{}\n", format_z(state.layer_z)).as_bytes());
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
