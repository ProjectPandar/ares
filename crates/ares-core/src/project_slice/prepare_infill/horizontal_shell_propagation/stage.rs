use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::CoordinateScale,
    project_slice::prepare_infill::{
        horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
        horizontal_shell_propagation::types::{
            PreparedPostHorizontalShellPropagation, WorkingFillRecord, WorkingObject,
            WorkingProject,
        },
        surface_type_detection::types::PreparedSurfaceTypeRecord,
    },
};

pub(super) fn prepare(
    prepared: PreparedPostHorizontalShellPromotion,
) -> Result<PreparedPostHorizontalShellPropagation, SliceError> {
    validate_alignment(&prepared);

    let mut working = build_working_graph(&prepared);
    super::propagate::project(&prepared, &mut working)?;
    Ok(commit_dirty(prepared, working))
}

fn build_working_graph(prepared: &PreparedPostHorizontalShellPromotion) -> WorkingProject {
    let objects = prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(object, traversal)| {
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
                    .map(|record| clone_working_record(record.as_ref()))
                    .collect()
            });
            WorkingObject { records }
        })
        .collect();
    WorkingProject { objects }
}

fn clone_working_record(record: Option<&PreparedSurfaceTypeRecord>) -> Option<WorkingFillRecord> {
    record.map(|record| WorkingFillRecord {
        fill_surfaces: record.fill_surfaces.clone(),
        dirty: false,
    })
}

fn commit_dirty(
    prepared: PreparedPostHorizontalShellPromotion,
    working: WorkingProject,
) -> PreparedPostHorizontalShellPropagation {
    let PreparedPostHorizontalShellPromotion {
        predecessor,
        mut objects,
    } = prepared;
    for (object, working_object) in objects.iter_mut().zip(working.objects) {
        let Some(working_records) = working_object.records else {
            continue;
        };
        for (original, working) in object.records.iter_mut().zip(working_records) {
            match (original, working) {
                (Some(original), Some(working)) if working.dirty => {
                    original.fill_surfaces = working.fill_surfaces;
                }
                (Some(_), Some(_)) | (None, None) => {}
                _ => unreachable!("validated O26 slots remain aligned"),
            }
        }
    }
    PreparedPostHorizontalShellPropagation {
        predecessor,
        objects,
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
        assert_eq!(
            (plan.source_object_index, plan.transform_index),
            input_object.identity()
        );

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
                _ => panic!("O26 slots remain aligned across the complete predecessor"),
            }
        }
    }
}
