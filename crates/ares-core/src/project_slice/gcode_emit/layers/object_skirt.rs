//! GCode.cpp:4456-4483,5325-5336: emit once before the first group member.

use super::{
    super::{motion, skirt},
    entry_builder::EntryBuilder,
};
use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) struct Input<'a, 'b> {
    pub(super) skirt: &'a Option<skirt::SkirtPlan>,
    pub(super) done: &'a mut [(usize, f64)],
    pub(super) object: usize,
    pub(super) layer: usize,
    pub(super) builder: &'a mut EntryBuilder<'b>,
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) slices: &'a [ExPolygon],
    pub(super) spacing: f64,
}

pub(super) fn emit(output: &mut Vec<u8>, state: &mut motion::EmitState, input: Input<'_, '_>) {
    let Some(plan) = input.skirt else { return };
    let Some(group) = plan.object_group(input.object) else {
        return;
    };
    let physical = &input.traversal.objects[input.object].physical_layers()[input.layer];
    let (count, last_z) = &mut input.done[group];
    if *count >= plan.layer_count || (*count > 0 && *last_z >= physical.print_z - 1e-4) {
        return;
    }
    let layer = skirt::SkirtLayer {
        index: *count,
        height_mm: f64::from((physical.print_z - *last_z) as f32),
    };
    *count += 1;
    *last_z = physical.print_z;
    let geometry = input
        .builder
        .geometry(input.object, input.layer, input.slices, input.spacing);
    let lower = (!geometry.lower_boundary_lines.is_empty())
        .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
    plan.emit_group(
        group,
        output,
        layer,
        geometry.view(input.traversal, input.object, input.layer, lower.as_ref()),
        state,
    );
}
