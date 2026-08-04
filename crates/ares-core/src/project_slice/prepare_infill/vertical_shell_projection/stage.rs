use crate::{
    SliceError,
    project_slice::prepare_infill::{
        vertical_shell_projection::{gather, types::VerticalShellProjectionObject},
        vertical_shells::PreparedPostVerticalShellCache,
    },
};

pub(super) fn project(
    prepared: &PreparedPostVerticalShellCache,
) -> Result<Vec<VerticalShellProjectionObject>, SliceError> {
    validate_alignment(prepared);
    prepared
        .objects
        .iter()
        .zip(&prepared.caches)
        .zip(&prepared.predecessor.objects)
        .map(|((object, cache_object), traversal)| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let input_object = &prelude.object;
            let (compensated, inputs) = input_object.as_parts();
            let (post_regions, lslices) = compensated.as_parts();
            let (plan, _, _) = post_regions.as_parts();
            let records = object
                .records
                .iter()
                .zip(&cache_object.records)
                .zip(inputs)
                .zip(&prelude.records)
                .enumerate()
                .map(|(index, (((record, cache), input), flow))| {
                    match (record, cache, input, flow) {
                        (Some(_), Some(_), Some(input), Some(flow)) => gather::project_record(
                            index,
                            gather::ProjectionInput {
                                caches: &cache_object.records,
                                layers: &plan.layers,
                                lslices,
                                options: input_object.region_options(input),
                                external_spacing: flow.external_spacing,
                            },
                        )
                        .map(Some),
                        (None, None, None, None) => Ok(None),
                        _ => unreachable!("validated O20 slots remain aligned"),
                    }
                })
                .collect::<Result<_, _>>()?;
            Ok(VerticalShellProjectionObject { records })
        })
        .collect()
}

fn validate_alignment(prepared: &PreparedPostVerticalShellCache) {
    assert_eq!(prepared.objects.len(), prepared.caches.len());
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for ((object, cache_object), traversal) in prepared
        .objects
        .iter()
        .zip(&prepared.caches)
        .zip(&prepared.predecessor.objects)
    {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        let (compensated, inputs) = input_object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        let count = object.records.len();
        assert_eq!(cache_object.records.len(), count);
        assert_eq!(inputs.len(), count);
        assert_eq!(prelude.records.len(), count);
        assert_eq!(plan.layers.len(), count);
        assert_eq!(lslices.len(), count);
        assert_eq!(regions.len(), 1);
        assert_eq!(plan.source_object_index, input_object.identity().0);
        assert_eq!(plan.transform_index, input_object.identity().1);
        for (index, (((record, cache), input), flow)) in object
            .records
            .iter()
            .zip(&cache_object.records)
            .zip(inputs)
            .zip(&prelude.records)
            .enumerate()
        {
            match (record, cache, input, flow) {
                (Some(_), Some(_), Some(input), Some(_)) => {
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
                (None, None, None, None) => {}
                _ => panic!("O20 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
