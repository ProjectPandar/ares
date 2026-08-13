use crate::{
    SliceError,
    project_slice::prepare_infill::{
        bridge_over_infill::types::BridgeCandidateObject,
        external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::super::{
    bridge_rewrite_areas::{UpperBridgeEnsuringInput, collect_bridge_rewrite_areas},
    internal_bridge_surfaces::build_internal_bridge_surfaces,
    internal_infill_rebuild::rebuild_internal_infills,
    internal_solid_recomposition::recompose_internal_solids,
    region_bridge_ensuring_areas::prepare_region_bridge_ensuring_areas,
    region_bridge_surface_commit::commit_region_bridge_surfaces,
};
use super::geometry_error;

pub(super) fn prepare(
    predecessor: &mut PreparedPostExternalSurfaces,
    candidate_objects: &[BridgeCandidateObject],
) -> Result<(), SliceError> {
    let horizontal = &mut predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let scale = traversal.scale;
    for (object_index, candidates) in candidate_objects.iter().enumerate() {
        let traversal_object = &traversal.objects[object_index];
        let prelude = &traversal_object
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let (_, inputs) = prelude.object.as_parts();
        let object = &mut horizontal.objects[object_index];
        for layer_index in 0..object.records.len() {
            let current = candidates
                .surfaces_by_layer
                .get(&layer_index)
                .map(Vec::as_slice);
            let upper = candidates.surfaces_by_layer.get(&(layer_index + 1));
            let upper_inputs = upper.map(|surfaces| {
                surfaces
                    .iter()
                    .map(|surface| UpperBridgeEnsuringInput {
                        surface,
                        solid_infill_flow: inputs[surface.source.layer_index]
                            .as_ref()
                            .expect("upper bridge source retains its perimeter input")
                            .solid_infill_flow,
                    })
                    .collect::<Vec<_>>()
            });
            let Some(areas) = collect_bridge_rewrite_areas(current, upper_inputs.as_deref(), scale)
                .map_err(geometry_error)?
            else {
                continue;
            };
            let (input, record) = match (&inputs[layer_index], &mut object.records[layer_index]) {
                (Some(input), Some(record)) => (input, record),
                (None, None) => continue,
                _ => unreachable!("scheduled bridge layer records remain aligned"),
            };
            let ensuring = prepare_region_bridge_ensuring_areas(
                &record.fill_surfaces,
                &areas.additional_ensuring_areas,
                input.solid_infill_flow,
                scale,
            )
            .map_err(geometry_error)?;
            let mut new_surfaces = rebuild_internal_infills(
                &record.fill_surfaces,
                &areas.cut_from_infill,
                &ensuring.additional_ensuring,
            )
            .map_err(geometry_error)?;
            new_surfaces.extend(
                build_internal_bridge_surfaces(
                    input.current.region_index,
                    &record.fill_surfaces,
                    current.unwrap_or(&[]),
                )
                .map_err(geometry_error)?,
            );
            new_surfaces.extend(
                recompose_internal_solids(
                    &record.fill_surfaces,
                    &ensuring.additional_ensuring,
                    &areas.cut_from_infill,
                )
                .map_err(geometry_error)?,
            );
            let original = std::mem::take(&mut record.fill_surfaces);
            record.fill_surfaces = commit_region_bridge_surfaces(original, &new_surfaces);
        }
    }
    Ok(())
}
