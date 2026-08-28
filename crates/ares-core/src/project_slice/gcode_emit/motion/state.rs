use super::{MotionOptions, arc, jerk, set_acceleration};

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct EmitState {
    pub(in crate::project_slice::gcode_emit) x: f64,
    pub(in crate::project_slice::gcode_emit) y: f64,
    pub(in crate::project_slice::gcode_emit) offset: (f64, f64),
    pub(in crate::project_slice::gcode_emit) scale_factor: f64,
    pub(in crate::project_slice::gcode_emit) travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) extrusion_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) options: MotionOptions,
    pub(in crate::project_slice::gcode_emit) layer_index: usize,
    pub(in crate::project_slice::gcode_emit) positioned: bool,
    pub(in crate::project_slice::gcode_emit) last_scaled_position: Option<(i64, i64)>,
    pub(in crate::project_slice::gcode_emit) last_feature: Option<&'static str>,
    pub(in crate::project_slice::gcode_emit) last_width: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_height: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) last_jerk: Option<f64>,
    pub(in crate::project_slice::gcode_emit) layer_z: f64,
    pub(in crate::project_slice::gcode_emit) retracted: bool,
    pub(in crate::project_slice::gcode_emit) wipe_path: Vec<arc::Point>,
    pub(in crate::project_slice::gcode_emit) wipe_start: Option<arc::Point>,
    pub(in crate::project_slice::gcode_emit) lifted: bool,
    pub(in crate::project_slice::gcode_emit) part_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) physical_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) overhang_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) overhang_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) pending_object_start: Option<(u32, [u8; 12])>,
    pub(in crate::project_slice::gcode_emit) tags: super::super::tags::Tags,
    pub(in crate::project_slice::gcode_emit) pending_layer_retract: bool,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct LayerGeometry<'a> {
    pub(in crate::project_slice::gcode_emit) internal_surfaces:
        &'a [crate::project_slice::region_slices::RegionSurface],
    pub(in crate::project_slice::gcode_emit) scale: crate::geometry::CoordinateScale,
    pub(in crate::project_slice::gcode_emit) previous_layer_boundary:
        Option<&'a crate::geometry::LineDistanceTree<'a>>,
}

pub(in crate::project_slice::gcode_emit) fn begin_layer(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    layer_index: usize,
    layer_z: f64,
    layer_height: f64,
) {
    state.layer_index = layer_index;
    state.last_height = Some(layer_height as f32);
    state.layer_z = layer_z;
    state.travel_feedrate = if layer_index == 0 {
        state.options.first_layer_travel_feedrate
    } else {
        state.options.travel_feedrate
    };
    let acceleration = match layer_index {
        0 => Some(state.options.initial_layer_acceleration),
        1 => Some(state.options.default_acceleration),
        _ => None,
    };
    if let Some(acceleration) = acceleration {
        set_acceleration(output, state, acceleration);
    }
    let jerk = if state.options.default_jerk <= 0.0 {
        0.0
    } else if layer_index == 0 && state.options.initial_layer_jerk > 0.0 {
        state.options.initial_layer_jerk
    } else {
        state.options.default_jerk
    };
    jerk::set(output, state, jerk);
}

pub(in crate::project_slice::gcode_emit) fn queue_object_start(
    state: &mut EmitState,
    label_id: u32,
    encoded_labels: [u8; 12],
) {
    state.pending_object_start = Some((label_id, encoded_labels));
}

pub(in crate::project_slice::gcode_emit) fn append_object_start(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    let Some((label_id, encoded_labels)) = state.pending_object_start.take() else {
        return;
    };
    output.extend_from_slice(
        format!("; start printing object, unique label id: {label_id}\nM624 ").as_bytes(),
    );
    output.extend_from_slice(&encoded_labels);
    output.push(b'\n');
}

pub(in crate::project_slice::gcode_emit) fn begin_path_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    destination_feature: &str,
    travel_distance: f64,
) {
    let acceleration = if state.options.default_acceleration == 0 {
        None
    } else if state.layer_index == 0 {
        (state.options.initial_layer_travel_acceleration > 0)
            .then_some(state.options.initial_layer_travel_acceleration)
    } else if travel_distance < state.options.retraction_minimum_travel {
        match destination_feature {
            "Overhang wall" => {
                (state.options.bridge_acceleration > 0).then_some(state.options.bridge_acceleration)
            }
            "Outer wall" => (state.options.outer_wall_acceleration > 0)
                .then_some(state.options.outer_wall_acceleration),
            _ => None,
        }
        .or((state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration))
    } else {
        (state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration)
    };
    set_acceleration(output, state, acceleration.unwrap_or(0));

    let jerk = if state.options.default_jerk <= 0.0 {
        0.0
    } else if state.layer_index == 0 {
        state.options.travel_jerk
    } else if travel_distance < state.options.retraction_minimum_travel
        && matches!(destination_feature, "Outer wall" | "Overhang wall")
        && state.options.outer_wall_jerk > 0.0
    {
        state.options.outer_wall_jerk
    } else {
        state.options.travel_jerk
    };
    jerk::set(output, state, jerk);
}
