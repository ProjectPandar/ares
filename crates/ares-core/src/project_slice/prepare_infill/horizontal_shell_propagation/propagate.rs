use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    project_slice::prepare_infill::{
        horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
        horizontal_shell_propagation::{
            PropagationEvent, gather, geometry, record_event,
            types::{NeighborOutcome, SOURCE_KINDS, SourceKind, WorkingObject, WorkingProject},
            window,
        },
    },
};

pub(super) fn project(
    prepared: &PreparedPostHorizontalShellPromotion,
    working: &mut WorkingProject,
) -> Result<(), SliceError> {
    for object_index in 0..prepared.objects.len() {
        propagate_object(prepared, &mut working.objects[object_index], object_index)?;
    }
    Ok(())
}

fn propagate_object(
    prepared: &PreparedPostHorizontalShellPromotion,
    working: &mut WorkingObject,
    object_index: usize,
) -> Result<(), SliceError> {
    for source_index in 0..prepared.objects[object_index].records.len() {
        propagate_source(prepared, working, object_index, source_index)?;
    }
    Ok(())
}

fn propagate_source(
    prepared: &PreparedPostHorizontalShellPromotion,
    working: &mut WorkingObject,
    object_index: usize,
    source_index: usize,
) -> Result<(), SliceError> {
    let object = &prepared.objects[object_index];
    let traversal = &prepared.predecessor.objects[object_index];
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let (_, inputs) = prelude.object.as_parts();
    let (Some(_), Some(source_input)) = (&object.records[source_index], &inputs[source_index])
    else {
        return Ok(());
    };
    record_event(PropagationEvent::RecordVisit {
        object: object_index,
        layer: source_index,
    });
    let options = prelude.object.region_options(source_input);
    if options.ensure_vertical_shell_thickness == ProcessEnsureVerticalShellThickness::EnsureAll {
        record_event(PropagationEvent::EnsureAllSkip {
            object: object_index,
            layer: source_index,
        });
        return Ok(());
    }
    for kind in SOURCE_KINDS {
        propagate_kind(prepared, working, [object_index, source_index], kind)?;
    }
    Ok(())
}

fn propagate_kind(
    prepared: &PreparedPostHorizontalShellPromotion,
    working: &mut WorkingObject,
    indices: [usize; 2],
    kind: SourceKind,
) -> Result<(), SliceError> {
    let [object_index, source_index] = indices;
    let object = &prepared.objects[object_index];
    let source_record = object.records[source_index]
        .as_ref()
        .expect("a propagated source is populated");
    let traversal = &prepared.predecessor.objects[object_index];
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input_object = &prelude.object;
    let (compensated, inputs) = input_object.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let source_input = inputs[source_index]
        .as_ref()
        .expect("a propagated source input is populated");
    let options = input_object.region_options(source_input);
    let count = window::shell_count(kind, options);
    if !window::source_enabled(count) {
        return Ok(());
    }
    record_event(PropagationEvent::SourceKindVisit {
        object: object_index,
        layer: source_index,
        kind,
    });
    let working_records = working
        .records
        .as_mut()
        .expect("a non-EnsureAll source activates the working object");
    let source = working_records[source_index]
        .as_ref()
        .expect("a populated source has a working record");
    let mut solid = gather::source_paths(source_record, &source.fill_surfaces, kind);
    #[cfg(test)]
    {
        let original = gather::source_paths(source_record, &source_record.fill_surfaces, kind);
        super::record_gather(super::GatherObservation {
            object: object_index,
            layer: source_index,
            kind,
            dirty_before_gather: source.dirty,
            path_count: solid.len(),
            path_digest: super::hooks::path_digest(&solid),
            original_path_digest: super::hooks::path_digest(&original),
        });
    }
    if solid.is_empty() {
        return Ok(());
    }

    for neighbor_index in window::indices(kind, source_index, working_records.len()) {
        if !window::includes(
            kind,
            [source_index, neighbor_index],
            &plan.layers,
            count,
            options,
        ) {
            break;
        }
        record_event(PropagationEvent::NeighborVisit {
            object: object_index,
            source: source_index,
            neighbor: neighbor_index,
            kind,
        });
        let outcome = {
            let neighbor_fill = working_records[neighbor_index]
                .as_ref()
                .map_or(&[][..], |record| record.fill_surfaces.as_slice());
            geometry::process_neighbor(
                geometry::NeighborContext {
                    scale: prepared.predecessor.scale,
                    source_input,
                    neighbor_input: inputs[neighbor_index].as_ref(),
                    options,
                    neighbor_fill,
                },
                &mut solid,
            )?
        };
        match outcome {
            NeighborOutcome::EmptyIntersection => {
                if geometry::should_stop_after_empty(options) {
                    break;
                }
            }
            NeighborOutcome::Rebuilt(fill_surfaces) => {
                let neighbor = working_records[neighbor_index]
                    .as_mut()
                    .expect("an absent neighbor cannot produce an intersection");
                neighbor.fill_surfaces = fill_surfaces;
                neighbor.dirty = true;
                record_event(PropagationEvent::Rebuild {
                    object: object_index,
                    source: source_index,
                    neighbor: neighbor_index,
                    kind,
                });
            }
        }
    }
    Ok(())
}
