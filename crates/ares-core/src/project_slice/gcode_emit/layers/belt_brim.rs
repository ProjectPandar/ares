//! `GCode.cpp`: apron bands, coincident brim and the orphan layer sweep.

use super::{entry::EntryGeometry, motion, schedule::LayerToPrint};
use crate::{
    SliceError,
    geometry::Point,
    project_slice::belt::brim::{Band, BeltBrimPlan},
};

pub(super) fn band(plan: Option<&BeltBrimPlan>, layer: LayerToPrint) -> (bool, Option<&Band>) {
    let Some(plan) = plan else {
        return (false, None);
    };
    if let Some(index) = layer.belt_prologue {
        (true, Some(&plan.prologue[index]))
    } else {
        (
            false,
            layer
                .object_layer
                .map(|index| &plan.by_layer[index])
                .filter(|band| !band.paths.is_empty()),
        )
    }
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    band: &Band,
    timelapse_context: super::super::timelapse::Context<'_>,
    (shift, layer): (Point, Option<([usize; 2], &EntryGeometry<'_>)>),
) -> Result<(), SliceError> {
    let scale = timelapse_context.traversal.scale;
    // `process_layer` installs its representative layer before the brim visit;
    // `process_belt_brim_layer` leaves `m_layer` null for the apron.
    let geometry = layer.map_or(
        motion::LayerGeometry {
            timelapse_context,
            nearest_seam_penalties: None,
            staggered_inner: false,
            internal_surfaces: &[],
            support_islands: None,
            scale,
            previous_curled: &[],
            previous_layer_boundary: None,
            avoid_crossing: motion::AvoidCrossingGeometry {
                layer_slices: &[],
                perimeter_spacing: 0.0,
                external_perimeter_width: 0.0,
                top_surfaces: &[],
                chunk_slices: &[],
                chunk_perimeter_spacing: 0.0,
            },
        },
        |([object, index], geometry)| geometry.view(timelapse_context, object, index, None),
    );
    motion::set_origin(state, (scale.unscale(shift.x()), scale.unscale(shift.y())));
    motion::emit_belt_brim(output, &band.paths, geometry, state)
}
