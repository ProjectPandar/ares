use crate::{
    SliceError,
    project_slice::prepare_infill::{
        vertical_shell_regularization::{regularize, types::VerticalShellRegularizationObject},
        vertical_shell_trimming::PreparedPostVerticalShellTrim,
    },
};

pub(super) fn regularize(
    prepared: &PreparedPostVerticalShellTrim,
) -> Result<Vec<VerticalShellRegularizationObject>, SliceError> {
    validate_alignment(prepared);
    prepared
        .objects
        .iter()
        .zip(&prepared.trims)
        .zip(&prepared.predecessor.objects)
        .map(|((object, trim_object), traversal)| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let records = object
                .records
                .iter()
                .zip(&trim_object.records)
                .zip(&prelude.object.records)
                .map(|((record, trim), input)| match (record, trim, input) {
                    (Some(_), Some(trim), Some(input)) => regularize::regularize_record(
                        trim,
                        input
                            .solid_infill_flow
                            .scaled_spacing(prepared.predecessor.scale),
                    )
                    .map(Some),
                    (None, None, None) => Ok(None),
                    _ => unreachable!("validated O22 slots remain aligned"),
                })
                .collect::<Result<_, _>>()?;
            Ok(VerticalShellRegularizationObject { records })
        })
        .collect()
}

fn validate_alignment(prepared: &PreparedPostVerticalShellTrim) {
    assert_eq!(prepared.objects.len(), prepared.caches.len());
    assert_eq!(prepared.objects.len(), prepared.projections.len());
    assert_eq!(prepared.objects.len(), prepared.trims.len());
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for ((((object, cache_object), projection_object), trim_object), traversal) in prepared
        .objects
        .iter()
        .zip(&prepared.caches)
        .zip(&prepared.projections)
        .zip(&prepared.trims)
        .zip(&prepared.predecessor.objects)
    {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        let (compensated, inputs) = input_object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        let count = object.records.len();
        assert_eq!(cache_object.records.len(), count);
        assert_eq!(projection_object.records.len(), count);
        assert_eq!(trim_object.records.len(), count);
        assert_eq!(inputs.len(), count);
        assert_eq!(plan.layers.len() * regions.len(), count);
        assert_eq!(lslices.len(), plan.layers.len());
        assert_eq!(plan.source_object_index, input_object.identity().0);
        assert_eq!(plan.transform_index, input_object.identity().1);
        for (index, ((((record, cache), projection), trim), input)) in object
            .records
            .iter()
            .zip(&cache_object.records)
            .zip(&projection_object.records)
            .zip(&trim_object.records)
            .zip(inputs)
            .enumerate()
        {
            match (record, cache, projection, trim, input) {
                (Some(_), Some(_), Some(_), Some(_), Some(input)) => {
                    assert_eq!(input_object.region_layer_record_index(input.current), index);
                    assert_eq!(input.planned_layer_index, input.current.layer_index);
                    assert_eq!(input.layer_id, plan.layers[input.current.layer_index].id);
                    assert_eq!(
                        input.region_id,
                        regions[input.current.region_index].as_parts().0
                    );
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                    assert_eq!(
                        (input.source_object_index, input.transform_index),
                        input_object.identity()
                    );
                }
                (None, None, None, None, None) => {}
                _ => panic!("O22 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
