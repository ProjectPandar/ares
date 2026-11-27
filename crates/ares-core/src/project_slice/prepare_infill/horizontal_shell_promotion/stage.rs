use crate::{
    CancelToken, SliceError,
    geometry::CoordinateScale,
    project_slice::prepare_infill::{
        horizontal_shell_promotion::{
            promote::{self, StagedDecision},
            types::PreparedPostHorizontalShellPromotion,
        },
        vertical_shell_assignment::PreparedPostVerticalShellAssignment,
    },
};

struct StagedObject {
    records: Vec<Option<StagedDecision>>,
}

pub(super) fn prepare(
    prepared: PreparedPostVerticalShellAssignment,
    cancel: &CancelToken,
) -> Result<PreparedPostHorizontalShellPromotion, SliceError> {
    cancel.check()?;
    validate_alignment(&prepared);
    let staged = stage_decisions(&prepared, cancel)?;

    let PreparedPostVerticalShellAssignment {
        predecessor,
        mut objects,
    } = prepared;
    for (object, staged_object) in objects.iter_mut().zip(staged) {
        for (record, decision) in object.records.iter_mut().zip(staged_object.records) {
            cancel.check()?;
            match (record, decision) {
                (Some(record), Some(decision)) => promote::commit(record, decision),
                (None, None) => {}
                _ => unreachable!("validated O25 slots remain aligned"),
            }
        }
    }
    Ok(PreparedPostHorizontalShellPromotion {
        predecessor,
        objects,
    })
}

fn stage_decisions(
    prepared: &PreparedPostVerticalShellAssignment,
    cancel: &CancelToken,
) -> Result<Vec<StagedObject>, SliceError> {
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(object, traversal)| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let input_object = &prelude.object;
            let (_, inputs) = input_object.as_parts();
            let records = object
                .records
                .iter()
                .zip(inputs)
                .map(|(record, input)| {
                    cancel.check()?;
                    match (record, input) {
                        (Some(_), Some(input)) => {
                            let raw = input_object
                                .region_options(input)
                                .extra_solid_infills
                                .0
                                .as_str();
                            promote::stage_decision(raw, input.planned_layer_index).map(Some)
                        }
                        (None, None) => Ok(None),
                        _ => unreachable!("validated O25 slots remain aligned"),
                    }
                })
                .collect::<Result<_, _>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn validate_alignment(prepared: &PreparedPostVerticalShellAssignment) {
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
    assert_eq!(prepared.predecessor.objects.len(), object_count);

    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let count = object.records.len();
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        let (compensated, inputs) = input_object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        assert_eq!(inputs.len(), count);
        assert_eq!(plan.layers.len() * regions.len(), count);
        assert_eq!(lslices.len(), plan.layers.len());
        assert_eq!(plan.source_object_index, input_object.identity().0);
        assert_eq!(plan.transform_index, input_object.identity().1);

        for (index, (record, input)) in object.records.iter().zip(inputs).enumerate() {
            match (record, input) {
                (Some(_), Some(input)) => {
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
                (None, None) => {}
                _ => panic!("O25 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
