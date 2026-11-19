mod approach;
mod dispatch;
mod restate;
mod retract;
mod route;

use super::PathProperties;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, append_object_start, features,
};

pub(in crate::project_slice::gcode_emit::motion) enum TowerTravelZ {
    Layer,
    Explicit(f64),
    Type2Entry,
}

/// `travel_to(erMixed)` accepts an explicit Z without `_extrude`'s Z restatement.
pub(in crate::project_slice::gcode_emit::motion) fn tower_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    first_scaled: (i64, i64),
    geometry: LayerGeometry<'_>,
    z: TowerTravelZ,
) -> Result<(), crate::SliceError> {
    let first_x = geometry.scale.unscale(first_scaled.0) + state.offset.0;
    let first_y = geometry.scale.unscale(first_scaled.1) + state.offset.1;
    let comment = if state.options.gcode_comments {
        " ; Travel to a Wipe Tower"
    } else {
        ""
    };
    let request = approach::Approach {
        first_scaled,
        first_x,
        first_y,
        feature: "Mixed",
        is_perimeter: false,
        comment,
        geometry,
        first_position: !state.positioned,
        layer_change_travel: state.layer_change_travel_pending && state.positioned,
        slope_start_z: None,
        target_z: match z {
            TowerTravelZ::Layer => state.layer_z,
            TowerTravelZ::Explicit(z) => z,
            TowerTravelZ::Type2Entry => state.type2_entry_z,
        },
        type2_entry: matches!(z, TowerTravelZ::Type2Entry),
        travel_distance: (first_x - state.x).hypot(first_y - state.y),
    };
    approach::emit_needs_travel(output, state, request)?;
    state.layer_change_travel_pending = false;
    Ok(())
}

pub(super) struct Request<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) force_no_extrusion: bool,
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    request: Request<'_>,
) -> Result<(), crate::SliceError> {
    let Request {
        first_scaled,
        force_no_extrusion,
        first_x,
        first_y,
        properties,
        geometry,
    } = request;
    let after_contour = std::mem::replace(
        &mut state.last_path_z_contoured,
        properties.contour.is_some(),
    );
    // `positioned` mirrors `GCodeWriter::m_is_current_pos_clear`: cleared by
    // custom g-code (timelapse/toolchange), set by every travel. The
    // `never_positioned` flag mirrors `!GCode::m_last_pos_defined` — true
    // only before the first generated travel — and drives the extra
    // `G1 Z{nominal_z}` re-statement that `_last_pos_undefined` triggers
    // (`GCode.cpp:6381-6386`), which a merely-uncleared position does not.
    let first_position = !state.positioned;
    let never_positioned = !state.last_position_defined;
    let layer_change_travel = state.layer_change_travel_pending && !first_position;
    let slope_start_z = properties
        .slope
        .filter(|slope| (slope.z_begin - slope.z_end).abs() > super::SOURCE_EPSILON_MM)
        .map(|slope| {
            state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * slope.z_begin
        });
    // `GCode.cpp:6359-6368`: `slope_need_z_travel =
    // m_writer.will_move_z(target_z)` — the writer's live nozzle z
    // (with the lift band), not the nominal layer z.
    let slope_needs_z_travel =
        slope_start_z.is_some_and(|target| approach::travel_will_move_z(state, target));
    let contour_z = properties
        .contour
        .map(|points| state.layer_z + geometry.scale.unscale(points[0].z));
    // GCode.cpp:7480-7482 restores nominal Z through travel_to_xyz.
    let travel_z = slope_start_z
        .or(contour_z)
        .or(after_contour.then_some(state.layer_z));
    let target_z = travel_z.unwrap_or(state.layer_z);
    // `GCode.cpp:9226-9227`: the writer's clear flag is not part of the condition, and the
    // change-layer flag counts whatever the writer position.
    let needs_travel = !state.last_position_defined
        || state.last_scaled_position != first_scaled
        || state.layer_change_travel_pending
        || slope_needs_z_travel;
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let (travel_set_layer_z, eager_lifted_travel, unclear_position_travel) = if needs_travel {
        let comment = if state.options.gcode_comments {
            let bridge = if features::is_bridge_feature(properties.feature) {
                " (bridge)"
            } else {
                ""
            };
            format!(
                " ; move to first {}{bridge} point",
                if force_no_extrusion {
                    "move inwards before retraction/seam"
                } else {
                    features::feature_description(properties.feature)
                }
            )
        } else {
            String::new()
        };
        approach::emit_needs_travel(
            output,
            state,
            approach::Approach {
                first_scaled,
                first_x,
                first_y,
                feature: properties.feature,
                is_perimeter: properties.is_perimeter,
                comment: &comment,
                geometry,
                first_position,
                layer_change_travel,
                slope_start_z: travel_z,
                target_z,
                type2_entry: false,
                travel_distance,
            },
        )?
    } else {
        (false, false, false)
    };
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
    // Upstream re-states `G1 Z{nominal_z}` after the approach when
    // `_last_pos_undefined` (`GCode.cpp:6381-6386`) — even when the split
    // already emitted `G1 Z{target_z}` (the double `G1 Z.2` first travel).
    // A merely-uncleared position (post-timelapse/toolchange) does NOT get
    // the re-statement. The retracted path below descends an eager-lifted
    // nozzle itself (`state.lifted && !travel_set_layer_z`).
    restate::restate(
        output,
        state,
        restate::RestateContext {
            is_sloped: properties.slope.is_some(),
            contour_z,
            never_positioned,
            travel_set_layer_z,
            eager_lifted_travel,
            unclear_position_travel,
        },
    );
    Ok(())
}
