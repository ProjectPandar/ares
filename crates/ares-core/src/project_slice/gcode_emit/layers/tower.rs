//! Tower travel uses the leading native LayerToPrint's geometry.
use super::super::{motion, timelapse, wipe_tower};
use super::{entry::EntryGeometry, entry_builder::EntryBuilder, schedule::LayerToPrint};
use crate::{
    SliceError, geometry::ExPolygon, project_slice::support_material::InstalledSupportLayer,
};

pub(super) enum Event {
    Initialize,
    Finalize { layer_count: usize },
    Change { tool: usize, finish_layer: bool },
}
pub(super) struct Input<'a, 'b> {
    pub(super) context: timelapse::Context<'a>,
    pub(super) entry: &'a LayerToPrint,
    pub(super) builder: &'a mut EntryBuilder<'b>,
    pub(super) support: Option<&'a InstalledSupportLayer>,
    pub(super) chunk_slices: &'a [ExPolygon],
    pub(super) spacing: f64,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    event: Event,
    input: Input<'_, '_>,
) -> Result<(), SliceError> {
    if state.wipe_tower.is_none() {
        return Ok(());
    }
    let entry = input.entry;
    let traversal = input.context.traversal;
    let geometry = if let Some(layer) = entry.object_layer {
        input
            .builder
            .geometry(entry.object_index, layer, input.spacing)
    } else {
        EntryGeometry {
            previous_curled: &[],
            top_surfaces: Vec::new(),
            lower_boundary_lines: Vec::new(),
            nearest_penalties: None,
            layer_slices: input
                .support
                .map_or_else(|| Vec::new().into(), |layer| layer.slices.clone().into()),
            internal_surfaces: Vec::new(),
            chunk_slices: input.chunk_slices.to_vec(),
            chunk_perimeter_spacing: input.spacing,
        }
    };
    let boundary = (!geometry.lower_boundary_lines.is_empty())
        .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
    let mut view = if let Some(layer) = entry.object_layer {
        geometry.view(traversal, entry.object_index, layer, boundary.as_ref())
    } else {
        geometry.view_raft(traversal.scale)
    };
    if entry.object_layer.is_none()
        && let Some(support) = input.support
    {
        view.support_islands = Some(motion::LayerAreas {
            scaled_default_perimeter_spacing: None,
            islands: &support.support_islands,
            tree_base: support.tree_base_areas.as_deref(),
        });
    }
    match event {
        Event::Initialize => wipe_tower::initialize(output, state, input.context, view),
        Event::Finalize { layer_count } => {
            wipe_tower::finalize(output, state, input.context, view, layer_count)
        }
        Event::Change { tool, finish_layer } => {
            wipe_tower::tool_change(output, state, (tool, finish_layer), input.context, view)
        }
    }
}
