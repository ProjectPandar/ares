use crate::{
    SliceError,
    project_slice::prepare_infill::{
        vertical_shell_filtering::{filter, types::VerticalShellTinyFilterObject},
        vertical_shell_regularization::PreparedPostVerticalShellRegularization,
    },
};

pub(super) fn filter(
    prepared: &PreparedPostVerticalShellRegularization,
) -> Result<Vec<VerticalShellTinyFilterObject>, SliceError> {
    validate_alignment(prepared);
    let scale = prepared.predecessor.scale;
    prepared
        .objects
        .iter()
        .zip(&prepared.trims)
        .zip(&prepared.regularizations)
        .zip(&prepared.predecessor.objects)
        .map(
            |(((object, trim_object), regularization_object), traversal)| {
                let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
                let input_object = &prelude.object;
                let (compensated, _) = input_object.as_parts();
                let (_, lslices) = compensated.as_parts();
                let records = object
                    .records
                    .iter()
                    .zip(&trim_object.records)
                    .zip(&regularization_object.records)
                    .zip(&prelude.records)
                    .enumerate()
                    .map(|(index, (((record, trim), regularization), flow))| {
                        match (record, trim, regularization, flow) {
                            (Some(record), Some(trim), Some(regularization), Some(flow)) => {
                                filter::filter_record(
                                    filter::RecordOperands {
                                        trim,
                                        regularization,
                                        current: record,
                                        previous_lslices: index
                                            .checked_sub(1)
                                            .map(|previous| lslices[previous].as_slice()),
                                        next_lslices: lslices.get(index + 1).map(Vec::as_slice),
                                    },
                                    flow.solid_infill_spacing,
                                    scale,
                                )
                                .map(Some)
                            }
                            (None, None, None, None) => Ok(None),
                            _ => unreachable!("validated O23 slots remain aligned"),
                        }
                    })
                    .collect::<Result<_, _>>()?;
                Ok(VerticalShellTinyFilterObject { records })
            },
        )
        .collect()
}

fn validate_alignment(prepared: &PreparedPostVerticalShellRegularization) {
    assert_eq!(prepared.objects.len(), prepared.caches.len());
    assert_eq!(prepared.objects.len(), prepared.projections.len());
    assert_eq!(prepared.objects.len(), prepared.trims.len());
    assert_eq!(prepared.objects.len(), prepared.regularizations.len());
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (((((object, cache), projection), trim), regularization), traversal) in prepared
        .objects
        .iter()
        .zip(&prepared.caches)
        .zip(&prepared.projections)
        .zip(&prepared.trims)
        .zip(&prepared.regularizations)
        .zip(&prepared.predecessor.objects)
    {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        let (compensated, inputs) = input_object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        let count = object.records.len();
        assert_eq!(cache.records.len(), count);
        assert_eq!(projection.records.len(), count);
        assert_eq!(trim.records.len(), count);
        assert_eq!(regularization.records.len(), count);
        assert_eq!(traversal.records.len(), count);
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
                    (((((record, cache), projection), trim), regularization), traversal_record),
                    input,
                ),
                flow,
            ),
        ) in object
            .records
            .iter()
            .zip(&cache.records)
            .zip(&projection.records)
            .zip(&trim.records)
            .zip(&regularization.records)
            .zip(&traversal.records)
            .zip(inputs)
            .zip(&prelude.records)
            .enumerate()
        {
            match (
                record,
                cache,
                projection,
                trim,
                regularization,
                traversal_record,
                input,
                flow,
            ) {
                (Some(_), Some(_), Some(_), Some(_), Some(_), Some(_), Some(input), Some(_)) => {
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
                (None, None, None, None, None, None, None, None) => {}
                _ => panic!("O23 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
