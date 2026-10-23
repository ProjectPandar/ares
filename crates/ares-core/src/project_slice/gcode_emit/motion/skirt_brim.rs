//! Brim/skirt loop emission (`GCode::extrude_entity("brim"/"skirt")`).

use super::arc;
use super::features::PathProperties;
use super::local_cursor;
use super::path;
use super::state::{EmitState, LayerGeometry};
use crate::geometry::Point;
use crate::project_slice::gcode_emit::motion::SkirtLoopFlow;

pub(in crate::project_slice::gcode_emit) fn emit_brim_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    state.adaptive_pa_flow = Some(flow.mm3_per_mm * state.current_brim_flow_ratio);
    state.adaptive_pa_emitted = false;
    // `GCode::extrude_entity("brim")` dispatches to `extrude_loop`, which
    // splits every non-perimeter loop at the point nearest the last
    // position (`GCode.cpp:5771`). The projection foot truncates in plate
    // coordinates, so split there and restore, exactly like the skirt
    // (`GCode::generate_skirt` `set_origin(unscale(Point(0,0)))`).
    let points = points.collect::<Vec<_>>();
    if points.len() >= 2 {
        let offset = (
            (state.offset.0 / geometry.scale.factor()).round() as i64,
            (state.offset.1 / geometry.scale.factor()).round() as i64,
        );
        let cursor = local_cursor(state, geometry);
        let plate_target = Point::new(cursor.x() + offset.0, cursor.y() + offset.1);
        let plate = points
            .iter()
            .map(|&(x, y)| Point::new(x + offset.0, y + offset.1))
            .collect::<Vec<_>>();
        let mut split =
            crate::project_slice::gcode_emit::skirt::split_at_nearest(&plate, plate_target)
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect::<Vec<_>>();
        // Same plate-frame seam-gap clip as the skirt
        // (`GCode.cpp:5804` truncates toward zero on plate ints).
        super::clip::clip_end(
            &mut split,
            state.current_seam_gap_mm() / geometry.scale.factor(),
        );
        path::emit(
            output,
            split
                .iter()
                .map(|point| (point.0 - offset.0, point.1 - offset.1)),
            PathProperties {
                contour: None,
                mm3_per_mm: flow.mm3_per_mm,
                width: flow.width,
                height: flow.height,
                feature: "Brim",
                is_perimeter: false,
                // Already clipped in the plate frame above.
                end_clip: 0.0,
                fitting: &[],
                slope: None,
                inside_loop: false,
            },
            geometry,
            state,
        )?;
        // `extrude_loop`'s wipe storage (GCode.cpp:5979-5991) concatenates
        // the loop's paths FORWARD (no reverse) — the brim retract's wipe
        // walks the ring from the seam. `path::emit` stored the reversed
        // per-path list; overwrite with the pre-clip split in order.
        state.wipe_path = split
            .iter()
            .map(|point| arc::Point {
                x: geometry.scale.unscale(point.0 - offset.0),
                y: geometry.scale.unscale(point.1 - offset.1),
            })
            .collect::<Vec<_>>();
        state.adaptive_pa_flow = None;
        state.adaptive_pa_emitted = false;
        return Ok(());
    }
    path::emit(
        output,
        points.into_iter(),
        PathProperties {
            contour: None,
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Brim",
            is_perimeter: false,
            end_clip: state.current_seam_gap_mm(),
            fitting: &[],
            slope: None,
            inside_loop: false,
        },
        geometry,
        state,
    )?;
    state.adaptive_pa_flow = None;
    state.adaptive_pa_emitted = false;
    Ok(())
}

/// Flow and clipping state for the skirt loop (`GCode.cpp:5804`, `Flow.hpp::Flow`).
pub(in crate::project_slice::gcode_emit) struct SkirtLoopParameters {
    pub(in crate::project_slice::gcode_emit) flow: SkirtLoopFlow,
    // The skirt seam gap is clipped by the caller on the plate-frame
    // polyline (`GCode.cpp:5804`); `preclipped` suppresses the local-frame
    // `end_clip` inside `path::emit`.
    pub(in crate::project_slice::gcode_emit) preclipped: bool,
}

pub(in crate::project_slice::gcode_emit) fn emit_skirt_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    SkirtLoopParameters { flow, preclipped }: SkirtLoopParameters,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    state.adaptive_pa_flow = Some(flow.mm3_per_mm);
    state.adaptive_pa_emitted = false;
    path::emit(
        output,
        points,
        PathProperties {
            contour: None,
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Skirt",
            is_perimeter: false,
            // `GCode::extrude_loop` clips every loop by the seam gap
            // (`GCode.cpp:5778-5790`) on the plate-frame polyline; the
            // skirt caller pre-clips in that frame, so `path::emit` must
            // not clip again.
            end_clip: if preclipped {
                0.0
            } else {
                state.options.seam_gap
            },
            fitting: &[],
            slope: None,
            inside_loop: false,
        },
        geometry,
        state,
    )?;
    state.adaptive_pa_flow = None;
    state.adaptive_pa_emitted = false;
    // `GCode.cpp:5979-5991` stores loop wipe paths forward so the wipe
    // wraps from the loop end back toward the path start.
    state.wipe_path.reverse();
    Ok(())
}
