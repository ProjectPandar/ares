mod adaptive_pa;
mod avoid_crossing;
mod constant;
pub(super) mod contour;
mod retraction;
pub(in crate::project_slice::gcode_emit) mod role_change;
mod start_travel;
pub(super) use start_travel::tower_travel;
mod travel_emit;
mod variable;

pub(in crate::project_slice::gcode_emit) use avoid_crossing::Boundary;
#[allow(unused_imports)]
pub(super) use avoid_crossing::build_boundary;
pub(super) use retraction::can_skip_retraction;
use retraction::inside_support_islands;

use super::{
    EmitState, LayerGeometry, arc, clip, extrusion, fan, features::PathProperties,
    features::role_id, format::axis as format_axis, overhang, set_accel_and_jerk,
};

pub(super) const SOURCE_EPSILON_MM: f64 = 1e-4;

pub(super) fn emit(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties<'_>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    emit_with_speed(output, points, (properties, None), geometry, state)
}

pub(super) fn emit_with_speed(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties_and_speed: (PathProperties<'_>, Option<f64>),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    if let Some(wipe_path) = emit_subpath(output, points, properties_and_speed, geometry, state)? {
        state.wipe_path = wipe_path;
    }
    Ok(())
}

pub(super) fn emit_subpath(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    (properties, speed): (PathProperties<'_>, Option<f64>),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<Option<Vec<arc::Point>>, crate::SliceError> {
    if let Some(points) = properties.contour {
        let points = contour::clipped(points, properties.end_clip / geometry.scale.factor());
        let properties = PathProperties {
            contour: Some(&points),
            end_clip: 0.0,
            ..properties
        };
        emit_inner(
            output,
            points.iter().map(|p| (p.x, p.y)),
            (properties, speed, false),
            geometry,
            state,
        )
    } else {
        emit_inner(output, points, (properties, speed, false), geometry, state)
    }
}

pub(super) fn emit_force_no_extrusion(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties<'_>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    if let Some(wipe_path) = emit_inner(output, points, (properties, None, true), geometry, state)?
    {
        state.wipe_path = wipe_path;
    }
    Ok(())
}

fn emit_inner(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    (properties, requested_speed, force_no_extrusion): (PathProperties<'_>, Option<f64>, bool),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<Option<Vec<arc::Point>>, crate::SliceError> {
    let role = role_id(properties.feature);
    let role_changed = state.last_extrusion_role != Some(role);
    let mut scaled_points = points.collect::<Vec<_>>();
    // Upstream clips the loop tail on `path.polyline`
    // (`GCode.cpp:5804`); for the skirt the caller pre-clips on the
    // plate-frame polyline (`emit_skirt_loop` `preclipped`).
    clip::clip_end(
        &mut scaled_points,
        properties.end_clip / geometry.scale.factor(),
    );
    let Some((&first_scaled, &last_scaled)) = scaled_points.first().zip(scaled_points.last())
    else {
        return Ok(None);
    };
    // Upstream computes segment lengths from scaled integer coordinates
    // (`Line::length() * SCALING_FACTOR`, `GCode.cpp:6997-6998`), not from
    // the unscaled mm coordinates. Computing from mm loses precision through
    // the double rounding of `unscale` before the distance.
    let segment_lengths = scaled_points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].0 - pair[0].0) as f64;
            let dy = (pair[1].1 - pair[0].1) as f64;
            (dx * dx + dy * dy).sqrt() * geometry.scale.factor()
        })
        .collect::<Vec<_>>();
    let mut local_points = scaled_points
        .into_iter()
        .map(|(x, y)| (geometry.scale.unscale(x), geometry.scale.unscale(y)))
        .collect::<Vec<_>>();
    // Upstream `clip_fitting_end` mutates `polyline.points.back()` to the
    // projected arc endpoint (`Polyline.cpp:714-718`) BEFORE
    // `extrude_path` stores `m_wipe.path = path.polyline.to_polyline()`
    // (`GCode.cpp:6111`) and `set_last_pos(path.last_point())`
    // (`GCode.cpp:7259`) — so the wipe path and last_pos both see the
    // projected endpoint, not the pre-clip tail.
    let mut fitting = properties.fitting.to_vec();
    let mut last_scaled = last_scaled;
    if properties.end_clip > 0.0 {
        let pre_projected = local_points.last().copied();
        arc::clip_fitting_end(&mut local_points, &mut fitting, geometry.scale);
        if let Some(&projected) = local_points.last()
            && Some(projected) != pre_projected
        {
            last_scaled = (
                (projected.0 / geometry.scale.factor()).round() as i64,
                (projected.1 / geometry.scale.factor()).round() as i64,
            );
        }
    }

    let (acceleration, configured_speed) = properties.kinematics(
        &state.options,
        state.layer_index,
        state.object_layer_over_raft,
        requested_speed,
    );
    // GCode.cpp:6615-6617 caps against the same effective flow as extrusion.
    let capped_speed = if state.options.max_volumetric_speed > 0.0 {
        configured_speed.min(
            state.options.max_volumetric_speed
                / extrusion::effective_flow_at_emission(properties, state),
        )
    } else {
        configured_speed
    };
    let original_speed = properties.resonance_avoided(&state.options, capped_speed);
    let estimated = overhang::estimate(overhang::EstimateRequest {
        points: &local_points,
        properties,
        geometry,
        options: &state.options,
        on_first_layer: state.on_first_layer,
        object_layer_over_raft: state.object_layer_over_raft,
        original_speed,
    });
    let path_overhang_fan_active = estimated
        .as_ref()
        .is_some_and(|estimate| estimate.fan_active);
    let processed = estimated
        .filter(|estimate| estimate.segmented)
        .map(|estimate| estimate.points);
    let points = processed.as_ref().map_or_else(
        || {
            local_points
                .iter()
                .map(|&(x, y)| {
                    (
                        x + state.origin.0 - state.extruder_offset.0,
                        y + state.origin.1 - state.extruder_offset.1,
                    )
                })
                .collect::<Vec<_>>()
        },
        |points| {
            points
                .iter()
                .map(|point| {
                    (
                        travel_emit::quantize_axis(
                            point.x + state.origin.0 - state.extruder_offset.0,
                        ),
                        travel_emit::quantize_axis(
                            point.y + state.origin.1 - state.extruder_offset.1,
                        ),
                    )
                })
                .collect::<Vec<_>>()
        },
    );
    // Source travels to the raw path start before overhang processing quantizes its points.
    let Some(&(first_local_x, first_local_y)) = local_points.first() else {
        return Ok(None);
    };
    let (first_x, first_y) = (
        first_local_x + state.origin.0 - state.extruder_offset.0,
        first_local_y + state.origin.1 - state.extruder_offset.1,
    );
    start_travel::emit(
        output,
        state,
        start_travel::Request {
            first_scaled,
            force_no_extrusion,
            first_x,
            first_y,
            properties,
            geometry,
        },
    )?;
    // Acceleration/jerk emit BEFORE the custom role-change hooks
    // (`GCode.cpp:6460-6464` before `:6745-6770`) so a hook can
    // override them.
    let jerk = properties.jerk(&state.options, state.layer_index);
    set_accel_and_jerk(output, state, acceleration, jerk, false);
    // `_extrude`'s custom role-change gcode fires AFTER the approach
    // travel/unretract, before the marker block (`GCode.cpp:6745-6770`);
    // context = extrusion_role/last_extrusion_role names +
    // `layer_num = m_layer_index + 1`, `layer_z = print_z`.
    if role_changed
        && (!state.options.role_change_machine.is_empty()
            || !state.options.role_change_process.is_empty()
            || !state.options.role_change_filament.is_empty())
    {
        role_change::emit(output, role, state);
    }
    if state.extrusion_role_markers && role_changed {
        output.extend_from_slice(format!(";_EXTRUSION_ROLE:{role}\n").as_bytes());
    }
    if role_changed {
        state.last_extrusion_role = Some(role);
    }
    let previous_extrusion_feedrate = state.extrusion_feedrate;
    state.extrusion_feedrate = processed
        .as_ref()
        .map_or(original_speed, |points| points[0].speed)
        * 60.0;
    // `GCode.cpp:7255-7257`: the lift-eligibility role snapshot excludes
    // gap fill — tracked separately from the `;TYPE:` feature tracker and
    // updated for EVERY emitted extrusion path, not only feature changes.
    if properties.feature != "Gap infill" {
        state.last_notgapfill_feature = Some(properties.feature);
    }
    // `GCode.cpp:6782–6802`: tower markers bypass the live width/height cache.
    let last_was_tower = state.last_feature == Some("Prime tower");
    if state.last_feature != Some(properties.feature) {
        let feature = state.tags.feature(properties.feature) + "\n";
        output.extend_from_slice(feature.as_bytes());
        state.last_feature = Some(properties.feature);
    }
    if last_was_tower || state.last_width != Some(properties.width) {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .width(&super::super::format_processor_float(f64::from(
                        properties.width
                    )))
            )
            .as_bytes(),
        );
        state.last_width = Some(properties.width);
    }
    if last_was_tower
        || state
            .last_height
            .is_none_or(|height| (height - properties.height).abs() > 0.000_1)
    {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .height(&super::super::format_processor_float(f64::from(
                        properties.height
                    )))
            )
            .as_bytes(),
        );
        state.last_height = Some(properties.height);
    }
    let evaluated = adaptive_pa::tag(output, state, properties, acceleration, role_changed);
    if processed.is_none() && !evaluated {
        adaptive_pa::ramp(
            output,
            state,
            properties,
            acceleration,
            adaptive_pa::Ramp::Constant {
                previous: previous_extrusion_feedrate,
                next: state.extrusion_feedrate,
            },
        );
    }
    extrusion::speed(output, state.extrusion_feedrate, properties);
    state.current_feedrate = state.extrusion_feedrate;
    // `GCode.cpp:7115`: sloped paths run through the same processed
    // `extrude_path` loop when `estimate` produced `new_points` — the
    // `sloped` flag only changes the per-segment emission to
    // `extrude_to_xyz` (`GCode.cpp:7242-7248`), the delta-60
    // `set_speed`/`last_set_speed` tracking is identical.
    if let Some(processed) = processed {
        variable::emit(variable::Emission {
            output,
            points: &points,
            wipe_points: &local_points,
            processed: &processed,
            original_speed,
            acceleration,
            force_no_extrusion,
            properties,
            timelapse_context: geometry.timelapse_context,
            state,
        })?;
        state.last_scaled_position = last_scaled;
        state.last_position_defined = true;
        if properties.contour.is_none() {
            state.scarf_z = Some(state.layer_z);
        }
        output.extend_from_slice(b";_EXTRUDE_END\n");
    } else if properties.contour.is_some() {
        contour::emit(
            output,
            &points,
            (properties, path_overhang_fan_active),
            geometry,
            state,
        )?;
        state.last_scaled_position = last_scaled;
        state.last_position_defined = true;
    } else if properties.slope.is_some() {
        // Non-processed sloped path (`GCode.cpp:6992`): constant speed F
        // with Z-interpolated segments.
        fan::update_for_constant_path(output, properties, path_overhang_fan_active, state);
        super::scarf::emit_segments(
            output,
            (&points, &local_points),
            properties,
            state,
            geometry.timelapse_context,
        )?;
        output.extend_from_slice(b";_EXTRUDE_END\n");

        state.last_scaled_position = last_scaled;
        state.last_position_defined = true;
    } else {
        constant::emit(constant::Emission {
            output,
            points: &points,
            local_points: &local_points,
            timelapse_context: geometry.timelapse_context,
            segment_lengths: &segment_lengths,
            fitting: &fitting,
            last_scaled,
            path_overhang_fan_active,
            force_no_extrusion,
            properties,
            state,
        })?;
    }
    Ok(Some(
        local_points
            .into_iter()
            .rev()
            .map(|(x, y)| arc::Point { x, y })
            .collect(),
    ))
}
