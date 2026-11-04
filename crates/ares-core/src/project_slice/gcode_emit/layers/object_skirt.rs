//! GCode.cpp:4456-4483,5325-5336: emit once before the first group member.

use super::{
    super::{motion, skirt},
    entry::EntryGeometry,
    entry_builder::EntryBuilder,
    raft_schedule::RaftSchedule,
};
use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    project_slice::support_material::InstalledSupportLayer,
};

pub(super) struct Input<'a, 'b> {
    pub(super) timelapse_context: super::super::timelapse::Context<'a>,
    pub(super) skirt: &'a Option<skirt::SkirtPlan>,
    pub(super) done: &'a mut [(usize, f64)],
    pub(super) object: usize,
    pub(super) copy: usize,
    pub(super) layer: Option<usize>,
    pub(super) builder: &'a mut EntryBuilder<'b>,
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) slices: &'a [ExPolygon],
    pub(super) spacing: f64,
    pub(super) support_layers: &'a [Vec<InstalledSupportLayer>],
    pub(super) raft: &'a Option<RaftSchedule>,
    pub(super) support_layer: Option<usize>,
    pub(super) raft_layer: Option<usize>,
    pub(super) print_z: f64,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    input: Input<'_, '_>,
) -> Result<(), crate::SliceError> {
    let Some(plan) = input.skirt else {
        return Ok(());
    };
    // `generate_object_skirt_group` (GCode.cpp:5976) returns nothing
    // unless skirt_type is stPerObject — stCombined emits once at the
    // plate level whatever the group membership.
    if !plan.per_object() {
        return Ok(());
    }
    let Some(group) = plan.instance_group((input.object, input.copy)) else {
        return Ok(());
    };
    let physical_z = input.layer.map_or(input.print_z, |layer| {
        input.traversal.objects[input.object].physical_layers()[layer].print_z
    });
    let (count, last_z) = &mut input.done[group];
    if *count >= plan.layer_count || (*count > 0 && *last_z >= physical_z - 1e-4) {
        return Ok(());
    }
    let layer = skirt::SkirtLayer {
        index: *count,
        height_mm: f64::from((physical_z - *last_z) as f32),
    };
    *count += 1;
    *last_z = physical_z;
    if let Some(object_layer) = input.layer {
        let geometry = input
            .builder
            .geometry(input.object, object_layer, input.spacing);
        let lower = (!geometry.lower_boundary_lines.is_empty())
            .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
        plan.emit_group(
            output,
            (group, layer),
            geometry.view(
                input.timelapse_context,
                input.object,
                object_layer,
                lower.as_ref(),
            ),
            state,
        )?;
    } else if let Some(raft_index) = input.raft_layer {
        // `GCode.cpp:5329-5335` calls generate_object_skirt_group for raft
        // entries; `:4398` selects first_layer and `:4311-4318` counts raft
        // layers toward skirt_height. Raft polygons/flow come from the
        // support-layer footprint path (`Print.cpp:2669-2696`).
        let raft = input.raft.as_ref().unwrap();
        let raft_plan = &raft.plans[input.object][raft_index];
        let raft_polygons = raft_plan
            .polygons
            .iter()
            .map(|polygon| ExPolygon::new(polygon.clone(), Vec::new()))
            .collect::<Vec<_>>();
        let geometry = EntryGeometry {
            previous_curled: &[],
            top_surfaces: Vec::new(),
            lower_boundary_lines: Vec::new(),
            nearest_penalties: None,
            layer_slices: raft_polygons.clone().into(),
            internal_surfaces: Vec::new(),
            chunk_slices: raft_polygons,
            chunk_perimeter_spacing: 0.0,
        };
        plan.emit_group(
            output,
            (group, layer),
            geometry.view_raft(input.timelapse_context),
            state,
        )?;
    } else if let Some(support_index) = input.support_layer {
        // `GCode.cpp:5329-5335` also invokes generate_object_skirt_group for
        // support-only entries; `:4398`/`:4311-4318` own first_layer and
        // skirt_height. Support geometry and skirt flow originate in the
        // occupied support footprints (`Print.cpp:2669-2696`) and group
        // emission (`GCode.cpp:4465-4485`).
        let support = &input.support_layers[input.object][support_index];
        let geometry = EntryGeometry {
            previous_curled: &[],
            top_surfaces: Vec::new(),
            lower_boundary_lines: Vec::new(),
            nearest_penalties: None,
            layer_slices: support.slices.clone().into(),
            internal_surfaces: Vec::new(),
            chunk_slices: input.slices.to_vec(),
            chunk_perimeter_spacing: input.spacing,
        };
        plan.emit_group(
            output,
            (group, layer),
            geometry.view_raft(input.timelapse_context),
            state,
        )?;
    }
    Ok(())
}
