use super::{MotionOptions, arc, set_accel_and_jerk, set_layer_acceleration_and_jerk};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum LiftMode {
    #[default]
    Normal,
    Slope,
    Spiral,
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct EmitState {
    pub(in crate::project_slice::gcode_emit) x: f64,
    pub(in crate::project_slice::gcode_emit) y: f64,
    pub(in crate::project_slice::gcode_emit) e_position: f64,
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
    pub(in crate::project_slice::gcode_emit) pending_lift: Option<LiftMode>,
    pub(in crate::project_slice::gcode_emit) part_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) physical_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) overhang_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) overhang_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) pending_object_start: Option<(u32, [u8; 12])>,
    pub(in crate::project_slice::gcode_emit) last_travel_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_start: Option<String>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_end: Option<String>,
    pub(in crate::project_slice::gcode_emit) tags: super::super::tags::Tags,
    pub(in crate::project_slice::gcode_emit) pending_layer_retract: bool,
    pub(in crate::project_slice::gcode_emit) layer_change_travel_pending: bool,
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
    state.layer_change_travel_pending = true;
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
        let jerk = if state.options.default_jerk <= 0.0 {
            0.0
        } else if layer_index == 0 && state.options.initial_layer_jerk > 0.0 {
            state.options.initial_layer_jerk
        } else {
            state.options.default_jerk
        };
        set_layer_acceleration_and_jerk(output, state, acceleration, jerk);
    }
}

pub(in crate::project_slice::gcode_emit) fn queue_object_start(
    state: &mut EmitState,
    label_id: u32,
    encoded_labels: [u8; 12],
) {
    state.pending_object_start = Some((label_id, encoded_labels));
}

pub(in crate::project_slice::gcode_emit) fn queue_exclude_start(
    state: &mut EmitState,
    text: String,
) {
    state.pending_exclude_start = Some(text);
}

/// Mirrors `GCode.cpp:5478-5494`: an unflushed start label is dropped
/// (empty instance) and only then does the end label get armed.
pub(in crate::project_slice::gcode_emit) fn queue_exclude_end(state: &mut EmitState, text: String) {
    if state.pending_exclude_start.take().is_none() {
        state.pending_exclude_end = Some(text);
    }
}

pub(in crate::project_slice::gcode_emit) fn append_exclude_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if let Some(text) = state.pending_exclude_end.take() {
        output.extend_from_slice(text.as_bytes());
        if !state.options.use_relative_e_distances && text.lines().any(|line| line == "G92 E0") {
            state.e_position = 0.0;
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn append_object_start(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    // `add_object_change_labels` writes the end label before the start label
    // (`GCodeWriter.cpp:1197-1201`).
    append_exclude_end(output, state);
    if let Some((label_id, encoded_labels)) = state.pending_object_start.take() {
        output.extend_from_slice(
            format!("; start printing object, unique label id: {label_id}\nM624 ").as_bytes(),
        );
        output.extend_from_slice(&encoded_labels);
        output.push(b'\n');
    }
    if let Some(text) = state.pending_exclude_start.take() {
        if !state.options.use_relative_e_distances && state.e_position.abs() > f64::EPSILON {
            output.extend_from_slice(b"G92 E0\n");
            state.e_position = 0.0;
        }
        output.extend_from_slice(text.as_bytes());
    }
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
    set_accel_and_jerk(output, state, acceleration.unwrap_or(0), jerk, true);
}
