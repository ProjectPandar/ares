use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::CoordinateScale,
    project_slice::prepare_infill::{
        horizontal_shell_promotion::{self, PreparedPostHorizontalShellPromotion},
        horizontal_shell_propagation::{
            PropagationEvent, record_commit, record_disposal, record_event,
            types::{
                PreparedPostHorizontalShellPropagation, WorkingFillRecord, WorkingObject,
                WorkingProject,
            },
        },
        surface_type_detection::types::PreparedSurfaceTypeRecord,
    },
};

pub(super) fn prepare(
    prepared: PreparedPostHorizontalShellPromotion,
) -> Result<PreparedPostHorizontalShellPropagation, SliceError> {
    validate_alignment(&prepared);
    #[cfg(test)]
    let original_snapshot = super::transaction_snapshot::fingerprint(&prepared);
    let mut working = build_working_graph(&prepared);
    if let Err(error) = super::propagate::project(&prepared, &mut working) {
        #[cfg(test)]
        {
            super::hooks::record_rollback_snapshot(original_snapshot);
            super::hooks::record_rollback_snapshot(super::transaction_snapshot::fingerprint(
                &prepared,
            ));
        }
        record_disposal();
        horizontal_shell_promotion::dispose(prepared);
        return Err(error);
    }
    Ok(commit_dirty(prepared, working))
}

fn build_working_graph(prepared: &PreparedPostHorizontalShellPromotion) -> WorkingProject {
    let objects = prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .enumerate()
        .map(|(object_index, (object, traversal))| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let (_, inputs) = prelude.object.as_parts();
            let active = object.records.iter().zip(inputs).any(|(record, input)| {
                matches!((record, input), (Some(_), Some(input)) if prelude
                    .object
                    .region_options(input)
                    .ensure_vertical_shell_thickness
                    != ProcessEnsureVerticalShellThickness::EnsureAll)
            });
            let records = active.then(|| {
                object
                    .records
                    .iter()
                    .enumerate()
                    .map(|(layer_index, record)| {
                        clone_working_record(object_index, layer_index, record.as_ref())
                    })
                    .collect()
            });
            WorkingObject { records }
        })
        .collect();
    WorkingProject { objects }
}

fn clone_working_record(
    object: usize,
    layer: usize,
    record: Option<&PreparedSurfaceTypeRecord>,
) -> Option<WorkingFillRecord> {
    record.map(|record| {
        record_event(PropagationEvent::FillClone { object, layer });
        WorkingFillRecord {
            fill_surfaces: record.fill_surfaces.clone(),
            dirty: false,
        }
    })
}

fn commit_dirty(
    prepared: PreparedPostHorizontalShellPromotion,
    working: WorkingProject,
) -> PreparedPostHorizontalShellPropagation {
    let PreparedPostHorizontalShellPromotion {
        predecessor,
        mut objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    for (object_index, (object, working_object)) in
        objects.iter_mut().zip(working.objects).enumerate()
    {
        let Some(working_records) = working_object.records else {
            continue;
        };
        for (layer_index, (original, working)) in
            object.records.iter_mut().zip(working_records).enumerate()
        {
            match (original, working) {
                (Some(original), Some(working)) if working.dirty => {
                    original.fill_surfaces = working.fill_surfaces;
                    record_commit();
                    record_event(PropagationEvent::DirtyCommit {
                        object: object_index,
                        layer: layer_index,
                    });
                }
                (Some(_), Some(_)) | (None, None) => {}
                _ => unreachable!("validated O26 slots remain aligned"),
            }
        }
    }
    PreparedPostHorizontalShellPropagation {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    }
}

fn validate_alignment(prepared: &PreparedPostHorizontalShellPromotion) {
    assert_eq!(
        prepared.predecessor.scale,
        CoordinateScale::from_printable_area(
            &prepared
                .predecessor
                .resolved
                .views
                .full
                .printer
                .remaining
                .printable_area,
        )
    );
    let object_count = prepared.objects.len();
    assert_eq!(prepared.caches.len(), object_count);
    assert_eq!(prepared.projections.len(), object_count);
    assert_eq!(prepared.trims.len(), object_count);
    assert_eq!(prepared.regularizations.len(), object_count);
    assert_eq!(prepared.filters.len(), object_count);
    assert_eq!(prepared.predecessor.objects.len(), object_count);

    for ((((((object, cache), projection), trim), regularization), filter), traversal) in prepared
        .objects
        .iter()
        .zip(&prepared.caches)
        .zip(&prepared.projections)
        .zip(&prepared.trims)
        .zip(&prepared.regularizations)
        .zip(&prepared.filters)
        .zip(&prepared.predecessor.objects)
    {
        let count = object.records.len();
        assert_eq!(cache.records.len(), count);
        assert_eq!(projection.records.len(), count);
        assert_eq!(trim.records.len(), count);
        assert_eq!(regularization.records.len(), count);
        assert_eq!(filter.records.len(), count);
        assert_eq!(traversal.records.len(), count);
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        let (compensated, inputs) = input_object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        assert_eq!(inputs.len(), count);
        assert_eq!(prelude.records.len(), count);
        assert_eq!(plan.layers.len(), count);
        assert_eq!(lslices.len(), count);
        assert_eq!(regions.len(), 1);
        assert_eq!(
            (plan.source_object_index, plan.transform_index),
            input_object.identity()
        );

        for (
            index,
            (
                (
                    (((((record, cache), projection), trim), regularization), filter),
                    traversal_record,
                ),
                input,
            ),
        ) in object
            .records
            .iter()
            .zip(&cache.records)
            .zip(&projection.records)
            .zip(&trim.records)
            .zip(&regularization.records)
            .zip(&filter.records)
            .zip(&traversal.records)
            .zip(inputs)
            .enumerate()
        {
            let flow = &prelude.records[index];
            match (
                record,
                cache,
                projection,
                trim,
                regularization,
                filter,
                traversal_record,
                input,
                flow,
            ) {
                (
                    Some(_),
                    Some(_),
                    Some(_),
                    Some(_),
                    Some(_),
                    Some(_),
                    Some(_),
                    Some(input),
                    Some(_),
                ) => {
                    assert_eq!(input.planned_layer_index, index);
                    assert_eq!(input.layer_id, plan.layers[index].id);
                    assert_eq!(input.current.layer_index, index);
                    assert_eq!(input.current.region_index, 0);
                    assert_eq!(input.region_id, regions[0].as_parts().0);
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                    assert_eq!(
                        (input.source_object_index, input.transform_index),
                        input_object.identity()
                    );
                }
                (None, None, None, None, None, None, None, None, None) => {}
                _ => panic!("O26 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
