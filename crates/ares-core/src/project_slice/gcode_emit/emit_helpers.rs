//! Helpers extracted from `gcode_emit.rs` — avoid-crossing boundary
//! cache, trailing-position scan, and the layer-end timelapse append.

use crate::SliceError;
use crate::geometry::ExPolygon;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

use super::{motion, timelapse};

/// The Z of the last G0/G1 move in the emitted prefix — what the
/// `GCodeWriter`'s Z position would be after emitting the same g-code
/// (feeds the `change_layer` `will_move_z` gate).
/// The avoid-crossing boundary covers every object's slices at the layer's
/// slice_z (`Layer::lslices`, `AvoidCrossingPerimeters.cpp:1100`): copies
/// of one source object are separate traversal objects here, but one
/// PrintObject upstream — their islands union into one boundary. The
/// cache holds one union per layer_index (all copies share the layout).
/// The avoid-crossing boundary covers every copy of the print object at
/// the layer's slice_z (`Layer::lslices`,
/// `AvoidCrossingPerimeters.cpp:1100`): copies of one source object are
/// separate traversal objects here, but one PrintObject upstream — their
/// islands union into one boundary. The cache is keyed by the source
/// object and layer index; distinct source objects never share a union.
pub(super) fn layer_boundary_slices_rc(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    cache: &mut std::collections::HashMap<(usize, usize), std::rc::Rc<[ExPolygon]>>,
) -> std::rc::Rc<[ExPolygon]> {
    let (source_object_index, _) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let Some(record) = traversal.objects[object_index]
        .records
        .get(layer_index)
        .and_then(Option::as_ref)
    else {
        return std::rc::Rc::from(
            traversal.objects[object_index]
                .slices(layer_index)
                .unwrap_or(&[]),
        );
    };
    let _ = record;
    cache
        .entry((source_object_index, layer_index))
        .or_insert_with(|| {
            // `Layer::lslices` carries the xy-compensated surfaces
            // (`PrintObjectSlice.cpp` applies `xy_contour_compensation`
            // upstream); the volume-level `occurrence_slices` are
            // uncompensated and would inflate the avoid-crossing
            // boundary by the compensation amount.
            let all = traversal.objects[object_index]
                .slices(layer_index)
                .unwrap_or(&[])
                .to_vec();
            std::rc::Rc::from(crate::geometry::union_expolygons(&all, &[]).unwrap_or_default())
        })
        .clone()
}

/// The final XY the start g-code left the nozzle at — mirrors
/// `GCodeWriter::m_pos.head<2>()` after the start g-code renders. X
/// and Y words apply independently (a move may update only one).
pub(super) fn trailing_gcode_xy(output: &[u8]) -> Option<(f64, f64)> {
    let text = std::str::from_utf8(output).ok()?;
    let mut x = None;
    let mut y = None;
    for line in text.lines() {
        let code = line.split(';').next()?.trim();
        let command = code.split_whitespace().next()?;
        if !matches!(command, "G0" | "G1") {
            continue;
        }
        for word in code.split_whitespace() {
            if let Some(value) = word.strip_prefix('X') {
                x = value.parse::<f64>().ok();
            } else if let Some(value) = word.strip_prefix('Y') {
                y = value.parse::<f64>().ok();
            }
        }
    }
    match (x, y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

pub(super) fn append_layer_end_timelapse(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    inserted: bool,
    traditional: bool,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    if inserted {
        return Ok(());
    }
    if traditional {
        // `GCode.cpp:5538-5546` `add_object_change_labels`: pending
        // object-end labels flush before the layer-end template.
        motion::append_exclude_end(output, state);
    }
    // Only the traditional path emits timelapse at layer end
    // (`GCode.cpp:5264-5300`). BBL renders it inside the next layer's
    // CHANGE_LAYER block (`GCode.cpp:5205-5210`); non-BBL non-traditional
    // already emitted it in the layer-change template
    // (`GCode.cpp:4667-4676`).
    if !traditional {
        return Ok(());
    }
    timelapse::append_and_track(output, state, context)
}

/// `GCodeWriter`/`GCodeProcessor` float formatting for emitted motion and
/// time values: up to 5 significant decimals, trailing zeros trimmed.
pub(in crate::project_slice) fn format_processor_float(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let precision = (5 - magnitude).max(0) as usize;
    let mut formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }
    formatted
}
