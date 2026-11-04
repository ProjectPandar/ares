use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, template, value};

#[derive(Clone, Copy)]
pub(super) struct TimelapseLayer {
    pub(super) index: usize,
    pub(super) z: f64,
    pub(super) max_z: f64,
}

#[derive(Clone, Copy)]
pub(super) struct Context<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) tool_ordering: &'a super::tool_ordering::ToolOrdering,
    pub(super) layer: TimelapseLayer,
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<footprint::FirstLayerBounds>,
}

/// The timelapse overlay split of the two upstream render sites: the
/// non-BBL layer-start render (`GCode.cpp:4667-4674`) overlays only
/// `layer_num`/`layer_z`/`max_layer_z`, while the `insert_timelapse_gcode`
/// lambda (`:5162-5167`, BBL layer-start and traditional layer-end
/// insert) additionally maps the physical extruder ids and the timelapse
/// park position.
#[derive(Clone, Copy, PartialEq)]
enum Overlay {
    LayerStart,
    InsertLambda,
    Inline,
}

/// The non-BBL layer-start render (`GCode.cpp:4667-4675`): emits the
/// timelapse template directly after `change_layer` WITHOUT the
/// `set_current_position_clear(false)` / `get_last_z_from_gcode` bookkeeping
/// — those live only in the `insert_timelapse_gcode` lambda
/// (`GCode.cpp:5171-5178`) used by the layer-end traditional insert and the
/// BBL layer-start insert. Clearing here would wrongly downgrade a deferred
/// spiral lift to the uncleared-position split (Qidi Q1 Pro emits a spiral
/// at every layer change because its position stays clear through this
/// render).
pub(super) fn append_untracked(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<(), SliceError> {
    append(output, state, context, Overlay::LayerStart)?;
    Ok(())
}

/// The `insert_timelapse_gcode` lambda path (layer-end traditional insert
/// and BBL layer-start): the rendered template parks the head, so upstream
/// marks the writer position unclear and adopts the template's last Z.
pub(super) fn append_and_track(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<(), SliceError> {
    append_generated(output, state, context, Overlay::InsertLambda).map(|_| ())
}

pub(super) fn append_inline(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<bool, SliceError> {
    append_generated(output, state, context, Overlay::Inline)
}

fn append_generated(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
    overlay: Overlay,
) -> Result<bool, SliceError> {
    let has_labels = state.layer_object_labels.is_some();
    if let Some(labels) = &state.layer_object_labels {
        labels.append_start(output, context.layer.index + 1);
    }
    let generated = append(output, state, context, overlay)?;
    if let Some(last_z) = generated {
        if let Some(z) = last_z {
            // `GCode.cpp:5173-5178`: the template's last Z is ADOPTED into
            // `m_pos(2)` — subsequent eager lifts target writer-domain
            // `m_pos.z() + hop` (`GCodeWriter.cpp:652-681`), so a `G1 Z5`
            // timelapse move changes the next lift's origin.
            state.record_z(z);
            // The template's absolute Z is writer-domain; compare against
            // the offset print z (`GCode.cpp:5171-5178` adopts it into
            // `m_pos` which already carries `z_offset`).
            state.lifted = z > context.layer.z + state.options.z_offset + f64::EPSILON;
            state.template_lifted = state.lifted;
        }
        // The timelapse g-code parks the head; upstream marks the writer
        // position unclear whenever the template is non-empty
        // (`GCode.cpp:5171-5178` runs on `!timelapse_gcode.empty()`, not on
        // a detected Z motion) so the next travel takes the separate
        // first-position form.
        state.positioned = false;
    }
    if let Some(labels) = &state.layer_object_labels {
        labels.append_end(output, context.layer.index + 1);
    }
    Ok(generated.is_some() || has_labels)
}

pub(super) fn append_traditional(
    enabled: bool,
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<bool, SliceError> {
    // `GCode.cpp:5455-5460`: one traditional insert per physical layer;
    // `layers/boundary.rs` resets the existing flag like `GCode.cpp:4653`.
    if !enabled || state.has_insert_timelapse_gcode {
        return Ok(false);
    }
    super::motion::prepare_traditional_timelapse(output, state);
    append_and_track(output, state, context)?;
    state.has_insert_timelapse_gcode = true;
    Ok(true)
}

fn append(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    Context {
        traversal,
        tool_ordering: _,
        layer,
        metadata,
        first_layer_bounds,
    }: Context<'_>,
    overlay: Overlay,
) -> Result<Option<Option<f64>>, SliceError> {
    let runtime = &traversal.resolved.views.runtime_gcode;
    let source = &runtime.time_lapse_gcode.0;
    if source.is_empty() {
        return Ok(None);
    }

    let mut config = super::placeholders::base_config(
        traversal,
        metadata,
        first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    // `ppi.update_from_gcodewriter(m_writer)` refreshes writer state
    // before each template (`GCode.cpp:3896`).
    super::motion::refresh_writer_state(&mut config, state);
    config.insert("layer_num", value::Value::number(layer.index as f64));
    config.insert("layer_z", value::Value::number(layer.z));
    config.insert("max_layer_z", value::Value::number(layer.max_z));
    if let Some((min_x, min_y, size_x, size_y)) = first_layer_bounds {
        config.insert(
            "first_layer_center_no_wipe_tower",
            value::Value::List(vec![
                value::Value::number(min_x + 0.5 * size_x),
                value::Value::number(min_y + 0.5 * size_y),
            ]),
        );
    }
    // `ToolOrdering::cal_most_used_extruder` (`ToolOrdering.cpp:955-981`,
    // `state.most_used_extruder`): the placeholder id is then
    // `physical_extruder_map.get_at(most_used)` — but ONLY on the
    // `insert_timelapse_gcode` lambda overlay (`GCode.cpp:5162-5167`);
    // the non-BBL layer-start render (`:4667-4674`) overlays only
    // layer_num/layer_z/max_layer_z, so those spellings stay unresolved
    // variables there.
    if overlay != Overlay::LayerStart {
        let most_used = insert_physical_extruder_ids(&mut config, traversal, state);
        let inline = overlay == Overlay::Inline;
        let position = if inline {
            None
        } else {
            safe_position(traversal, most_used)
        };
        config.insert("timelapse_inline_photo", value::Value::option_bool(inline));
        config.insert(
            "farthest_point_timelapse_enabled",
            value::Value::option_bool(state.farthest_point.enabled),
        );
        config.insert(
            "has_timelapse_safe_pos",
            value::Value::option_bool(position.is_some()),
        );
        let (x, y) = position.unwrap_or_default();
        config.insert("timelapse_pos_x", value::Value::number(f64::from(x)));
        config.insert("timelapse_pos_y", value::Value::number(f64::from(y)));
    }

    // Write-back validation precedes emission; render or validation
    // failure emits only the inline report (`GCode.cpp:3920-3933`).
    let mut failure_error = String::new();
    let rendered =
        match template::render_for_filament(source, &mut config, state.extruders.active_id()) {
            Ok(rendered) => Some(rendered),
            Err(error) => {
                failure_error = error.to_string();
                state.record_template_failure("timelapse_gcode", &failure_error);
                None
            }
        };
    let last_z = rendered.as_deref().and_then(last_motion_z);
    let name = "timelapse_gcode";
    let failed = rendered.as_deref().is_none_or(|text| {
        super::motion::apply_writer_state(name, &config, text, state)
            .map_err(|error| failure_error = error)
            .is_err()
    });
    if failed {
        let error = failure_error;
        output.extend_from_slice(
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
            .as_bytes(),
        );
        // The nonempty failure report still leaves the position unclear
        // (`GCode.cpp:5167-5172` tracks the emitted block like any other
        // timelapse output).
        return Ok(Some(None));
    }
    output.extend_from_slice(rendered.unwrap().as_bytes());
    output.push(b'\n');
    // Outer `Some` = the template rendered and emitted (position now
    // unclear); inner `Some` = the rendered g-code ended on a Z motion.
    Ok(Some(last_z))
}

/// `GCode.cpp:5162-5163` (and the wrapping-detection twin
/// `:5232-5233`): `most_used_physical_extruder_id` maps the layer-count
/// winner (`ToolOrdering.cpp:955-981`, `state.most_used_extruder`)
/// through `physical_extruder_map.get_at`, while
/// `curr_physical_extruder_id` maps the writer's current
/// `extruder_id()` — `Extruder.cpp:21-27` computes it as
/// `filament_map.get_at(filament_id) - 1` (out-of-range filament id
/// falls back to 0), i.e. the active filament's map group.
pub(in crate::project_slice::gcode_emit) fn insert_physical_extruder_ids(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
    state: &super::motion::EmitState,
) -> Option<usize> {
    let runtime = &traversal.resolved.views.runtime_gcode;
    let map_at = |index: usize| {
        runtime
            .physical_extruder_map
            .0
            .get(index)
            .or_else(|| runtime.physical_extruder_map.0.first())
            .map_or(0, |value| value.0)
    };
    let most_used = map_at(state.most_used_extruder);
    let curr = map_at(state.extruders.active_physical());
    config.insert(
        "most_used_physical_extruder_id",
        value::Value::number(most_used as f64),
    );
    config.insert(
        "curr_physical_extruder_id",
        value::Value::number(curr as f64),
    );
    Some(state.most_used_extruder)
}

pub(super) fn last_motion_z(gcode: &str) -> Option<f64> {
    gcode.lines().rev().find_map(|line| {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        matches!(
            code.split_ascii_whitespace().next(),
            Some("G0" | "G1" | "G2" | "G3")
        )
        .then(|| {
            code.split_ascii_whitespace()
                .skip(1)
                .find_map(|word| word.strip_prefix('Z')?.parse().ok())
        })
        .flatten()
    })
}

fn safe_position(
    traversal: &PreparedPostClassicTraversal,
    picture_extruder: Option<usize>,
) -> Option<(i32, i32)> {
    picture_extruder.and_then(|extruder| super::timelapse_pos::position(traversal, extruder))
}
