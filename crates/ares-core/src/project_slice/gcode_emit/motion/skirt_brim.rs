//! Brim/skirt loop emission (`GCode::extrude_entity("brim"/"skirt")`).

use super::arc;
use super::features::PathProperties;
use super::local_cursor;
use super::path;
use super::state::{EmitState, LayerGeometry};
use super::{extrusion, loop_paths, options, scarf};
use crate::geometry::Point;
use crate::project_slice::gcode_emit::motion::SkirtLoopFlow;

pub(in crate::project_slice) fn emit_brim_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
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
        let split = crate::project_slice::gcode_emit::skirt::split_at_nearest(&plate, plate_target);
        path::emit(
            output,
            split
                .iter()
                .map(|point| (point.x() - offset.0, point.y() - offset.1)),
            PathProperties {
                mm3_per_mm: flow.mm3_per_mm,
                width: flow.width,
                height: flow.height,
                feature: "Brim",
                is_perimeter: false,
                end_clip: state.options.seam_gap,
                fitting: &[],
                slope: None,
            },
            geometry,
            state,
        );
        // `extrude_loop`'s wipe storage (GCode.cpp:5979-5991) concatenates
        // the loop's paths FORWARD (no reverse) — the brim retract's wipe
        // walks the ring from the seam. `path::emit` stored the reversed
        // per-path list; overwrite with the pre-clip split in order.
        state.wipe_path = split
            .iter()
            .map(|point| arc::Point {
                x: geometry.scale.unscale(point.x() - offset.0) + state.origin.0
                    - state.extruder_offset.0,
                y: geometry.scale.unscale(point.y() - offset.1) + state.origin.1
                    - state.extruder_offset.1,
            })
            .collect::<Vec<_>>();
        return;
    }
    path::emit(
        output,
        points.into_iter(),
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Brim",
            is_perimeter: false,
            end_clip: state.options.seam_gap,
            fitting: &[],
            slope: None,
        },
        geometry,
        state,
    );
}

pub(in crate::project_slice) fn emit_skirt_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    path::emit(
        output,
        points,
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Skirt",
            is_perimeter: false,
            // `GCode::extrude_loop` clips every loop by the seam gap
            // (`GCode.cpp:5778-5790`); `path::emit` clips in the plate
            // frame to match upstream's origin-shifted coordinates.
            end_clip: state.options.seam_gap,
            fitting: &[],
            slope: None,
        },
        geometry,
        state,
    );
    // `GCode.cpp:5979-5991` stores loop wipe paths forward so the wipe
    // wraps from the loop end back toward the path start.
    state.wipe_path.reverse();
    if let Ok(path) = std::env::var("ARES_DUMP_FULLLOOP") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let mut scaled = state
                .wipe_path
                .iter()
                .map(|point| {
                    let x = ((point.x - state.offset.0) / state.scale_factor).round() as i64;
                    let y = ((point.y - state.offset.1) / state.scale_factor).round() as i64;
                    (x, y)
                })
                .collect::<Vec<_>>();
            // the wipe stores the path in reverse emission order; dump in
            // forward path order like upstream's `this->path.points`
            scaled.reverse();
            for (x, y) in scaled {
                let _ = writeln!(file, "FL ({x},{y})");
            }
            let _ = writeln!(file, "FL_END");
        }
    }
}
