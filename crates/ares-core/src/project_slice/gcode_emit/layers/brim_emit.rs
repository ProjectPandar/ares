//! Brim emission — the brim prints on object 0's first layer before the
//! object content (`GCode.cpp` emits it ahead of the layer's object
//! entries).

use super::super::{brim, motion};
use super::entry_builder::EntryBuilder;
use crate::geometry::ExPolygon;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

#[expect(
    clippy::too_many_arguments,
    reason = "the brim branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    brim: &Option<brim::BrimPlan>,
    object_index: usize,
    layer_index: usize,
    entry_builder: &mut EntryBuilder<'_>,
    traversal: &PreparedPostClassicTraversal,
    chunk_slices: &[ExPolygon],
    chunk_perimeter_spacing: f64,
) {
    if object_index == 0
        && layer_index == 0
        && let Some(plan) = brim
    {
        let geometry = entry_builder.geometry(
            object_index,
            layer_index,
            chunk_slices,
            chunk_perimeter_spacing,
        );
        let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
            .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
        plan.emit(
            output,
            geometry.view(
                traversal,
                object_index,
                layer_index,
                lower_boundary.as_ref(),
            ),
            state,
        );
    }
}
