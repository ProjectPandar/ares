//! `GCode::PlaceholderParserIntegration` writer refresh and write-back.

use super::EmitState;
use crate::project_slice::gcode_emit::value::{Config, Value};

const MAXIMUM_EXTRUDER_NUMBER: usize = 64;

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => Some(*n),
        Value::Integer(n) => Some(*n as f64),
        _ => None,
    }
}

/// `GCode.cpp:1663-1697`: every registered extruder contributes private
/// counters; the statistics vectors include zero-filled unused tool slots.
pub(in crate::project_slice::gcode_emit) fn refresh_writer_state(
    config: &mut Config,
    state: &EmitState,
) {
    state.initial_filaments.insert(config);
    config.insert(
        "hold_chamber_temp_for_flat_print",
        Value::option_bool(state.hold_chamber_temp_for_flat_print),
    );
    config.insert(
        "first_layer_print_convex_hull",
        Value::List(
            state
                .first_layer_hull
                .iter()
                .map(|point| {
                    // ConfigOptionPoints renders indexed values via to_string(Vec2d).
                    let x = super::format::ostream(point.x() as f64 * state.scale_factor);
                    let y = super::format::ostream(point.y() as f64 * state.scale_factor);
                    Value::String(format!("[{x}, {y}]"))
                })
                .collect(),
        ),
    );
    if let Some(tower) = &state.wipe_tower {
        tower.insert_placeholders(config);
    }
    if state.extruder_selected {
        crate::project_slice::gcode_emit::placeholders::refresh_filament_aliases(
            config,
            state.extruders.active_id(),
        );
        for name in ["current_extruder", "filament_extruder_id"] {
            config.insert(name, Value::Integer(state.extruders.active_id() as i64));
        }
        config.insert(
            "current_hotend",
            Value::Integer(i64::from(state.current_hotend)),
        );
    }
    config.insert(
        "position",
        Value::List(vec![
            Value::number(state.x),
            Value::number(state.y),
            Value::number(state.writer_z.unwrap_or(0.0)),
        ]),
    );
    config.insert("zhop", Value::number(state.lifted_amount));
    let sharing = state.options.single_extruder_multi_material;
    let has_extruders = !state.extruders.ids().is_empty();
    let vector_width = if has_extruders {
        MAXIMUM_EXTRUDER_NUMBER
    } else {
        0
    };
    let mut positions = (has_extruders && !state.options.use_relative_e_distances)
        .then(|| vec![Value::number(0.0); MAXIMUM_EXTRUDER_NUMBER]);
    let mut retracted = vec![Value::number(0.0); vector_width];
    let mut extra = vec![Value::number(0.0); vector_width];
    let mut volumes = vec![Value::number(0.0); state.extruders.vector_size()];
    let mut weights = vec![Value::number(0.0); state.extruders.vector_size()];
    let mut total_volume = 0.0;
    let mut total_weight = 0.0;
    for &id in state.extruders.ids() {
        let private = state.extruders.read(id, &state.extruder, sharing);
        if let Some(positions) = &mut positions {
            positions[id] = Value::number(private.position);
        }
        retracted[id] = Value::number(private.retracted);
        extra[id] = Value::number(private.restart_extra);
        let (volume, weight) = state.extruders.statistics(
            id,
            &state.extruder,
            (state.options.filament_area, state.options.filament_density),
            sharing,
        );
        volumes[id] = Value::number(volume);
        weights[id] = Value::number(weight);
        total_volume += volume;
        total_weight += weight;
    }
    if has_extruders {
        config.insert("e_retracted", Value::List(retracted));
        config.insert("e_restart_extra", Value::List(extra));
    }
    config.insert("extruded_volume", Value::List(volumes));
    config.insert("extruded_weight", Value::List(weights));
    config.insert("extruded_volume_total", Value::number(total_volume));
    config.insert("extruded_weight_total", Value::number(total_weight));
    if let Some(positions) = positions {
        config.insert("e_position", Value::List(positions));
    }
}

fn validate_vector_size(
    name: &str,
    config: &Config,
    key: &str,
    expected: usize,
    state: &mut EmitState,
) -> Result<(), String> {
    let Some(Value::List(list)) = config.get(key) else {
        return Ok(());
    };
    if list.len() != expected {
        let error = format!("\"{key}\" output variable must not be resized by the script.");
        state.record_template_failure(name, &error);
        return Err(error);
    }
    Ok(())
}

/// Writer write-back (`GCode.cpp:3903-3915`): assigned `position` and
/// E counters propagate back into the writer so subsequent output uses
/// the overridden values.
/// Returns an error when a write-back raised a template error — the caller
/// emits the upstream inline failure block and skips the remaining
/// templates (`GCode.cpp:3920-3933`).
pub(in crate::project_slice::gcode_emit) fn apply_writer_state(
    name: &str,
    config: &Config,
    gcode: &str,
    state: &mut EmitState,
) -> Result<(), String> {
    // Vector-size validation first (`GCode.cpp:3898` calls
    // `validate_output_vector_variables`, `GCode.cpp:1702-1713`): only
    // `position` (exactly 3) and the 64-wide E vectors are checked.
    validate_vector_size(name, config, "position", 3, state)?;
    if !state.extruders.ids().is_empty() {
        if !state.options.use_relative_e_distances {
            validate_vector_size(name, config, "e_position", MAXIMUM_EXTRUDER_NUMBER, state)?;
        }
        for key in ["e_retracted", "e_restart_extra"] {
            validate_vector_size(name, config, key, MAXIMUM_EXTRUDER_NUMBER, state)?;
        }
    }
    // `GCode.cpp:5362-5366`: after validation, before the position write-back.
    super::invalidate_after_custom_gcode(gcode, state);
    if let Some(Value::List(position)) = config.get("position")
        && let [x, y, z] = position.as_slice()
        && let (Some(x), Some(y), Some(z)) = (numeric(x), numeric(y), numeric(z))
    {
        // `ppi.position != pos` is an exact 3-component compare — ANY
        // difference (including Z-only) triggers `set_position` +
        // `set_last_pos(gcode_to_point)` (`GCode.cpp:3901-3907`), and
        // `gcode_to_point` TRUNCATES the new coordinate
        // (`Point.hpp:665`).
        // `ppi.position != pos` compares against the writer's live
        // position — never-yet-assigned Z reads as the initial 0
        // (`GCodeWriter.hpp:165`).
        if x != state.x || y != state.y || state.writer_z.unwrap_or(0.0) != z {
            state.x = x;
            state.y = y;
            // `gcode_to_point` adds the extruder offset only with an
            // active extruder (`GCode.cpp:8138-8144`) — machine-start
            // and the non-BBL filament-start run before selection.
            let offset = if state.extruder_selected {
                state.extruder_offset
            } else {
                (0.0, 0.0)
            };
            state.last_scaled_position = (
                ((x - state.origin.0 + offset.0) / state.scale_factor) as i64,
                ((y - state.origin.1 + offset.1) / state.scale_factor) as i64,
            );
            state.last_position_defined = true;
        }
        state.writer_z = Some(z);
    }
    let sharing = state.options.single_extruder_multi_material;
    for slot in 0..state.extruders.ids().len() {
        let id = state.extruders.ids()[slot];
        let current = state.extruders.read(id, &state.extruder, sharing);
        if !state.options.use_relative_e_distances
            && let Some(position) = config
                .get("e_position")
                .and_then(|value| value.index(id))
                .and_then(numeric)
            && !is_approx(position, current.position)
        {
            state
                .extruders
                .write_position(id, position, &mut state.extruder, sharing);
        }
        let retracted = config
            .get("e_retracted")
            .and_then(|value| value.index(id))
            .and_then(Value::as_number);
        let extra = config
            .get("e_restart_extra")
            .and_then(|value| value.index(id))
            .and_then(Value::as_number);
        if let (Some(retracted), Some(extra)) = (retracted, extra)
            && (!is_approx(retracted, current.retracted)
                || !is_approx(extra, current.restart_extra))
        {
            if retracted < -1e-4 {
                state.record_template_failure(name, "Custom G-code reports negative z_retracted.");
                return Err("Custom G-code reports negative z_retracted.".to_string());
            }
            if extra < -1e-4 {
                state.record_template_failure(
                    name,
                    "Custom G-code reports negative z_restart_extra.",
                );
                return Err("Custom G-code reports negative z_restart_extra.".to_string());
            }
            state
                .extruders
                .write_retracted(id, (retracted, extra), &mut state.extruder, sharing);
        }
    }
    Ok(())
}

/// `libslic3r.h:307-309`: fabs(a - b) < EPSILON.
fn is_approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}
