use crate::{
    SliceError,
    geometry::CoordinateScale,
    project_slice::prepare_infill::{
        vertical_shell_assignment::{
            assign::{self, StagedAssignment},
            types::PreparedPostVerticalShellAssignment,
        },
        vertical_shell_filtering::{self, PreparedPostVerticalShellFiltering},
    },
};

struct StagedObject {
    records: Vec<Option<StagedAssignment>>,
}

pub(super) fn prepare(
    prepared: PreparedPostVerticalShellFiltering,
) -> Result<PreparedPostVerticalShellAssignment, SliceError> {
    validate_alignment(&prepared);
    let staged = match stage_assignments(&prepared) {
        Ok(staged) => staged,
        Err(error) => {
            vertical_shell_filtering::dispose(prepared);
            return Err(error);
        }
    };

    let PreparedPostVerticalShellFiltering {
        predecessor,
        mut objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    for (object, staged_object) in objects.iter_mut().zip(staged) {
        for (record, staged_record) in object.records.iter_mut().zip(staged_object.records) {
            match (record, staged_record) {
                (Some(record), Some(staged_record)) => assign::commit(record, staged_record),
                (None, None) => {}
                _ => unreachable!("validated O24 slots remain aligned"),
            }
        }
    }
    Ok(PreparedPostVerticalShellAssignment {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    })
}

fn stage_assignments(
    prepared: &PreparedPostVerticalShellFiltering,
) -> Result<Vec<StagedObject>, SliceError> {
    prepared
        .objects
        .iter()
        .zip(&prepared.filters)
        .map(|(object, filter_object)| {
            let records = object
                .records
                .iter()
                .zip(&filter_object.records)
                .map(|(record, filter)| match (record, filter) {
                    (Some(record), Some(filter)) => assign::stage_record(record, filter).map(Some),
                    (None, None) => Ok(None),
                    _ => unreachable!("validated O24 slots remain aligned"),
                })
                .collect::<Result<_, _>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn validate_alignment(prepared: &PreparedPostVerticalShellFiltering) {
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
    assert_eq!(prepared.objects.len(), prepared.caches.len());
    assert_eq!(prepared.objects.len(), prepared.projections.len());
    assert_eq!(prepared.objects.len(), prepared.trims.len());
    assert_eq!(prepared.objects.len(), prepared.regularizations.len());
    assert_eq!(prepared.objects.len(), prepared.filters.len());
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
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
        assert_eq!(plan.source_object_index, input_object.identity().0);
        assert_eq!(plan.transform_index, input_object.identity().1);

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
                _ => panic!("O24 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
