use crate::{
    SliceError, project_slice::prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
};

use super::{cache, types::VerticalShellCacheObject};

pub(super) fn project(
    prepared: &PreparedPostFillSurfacePreparation,
) -> Result<Vec<VerticalShellCacheObject>, SliceError> {
    validate_alignment(prepared);
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(object, traversal)| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let input_object = &prelude.object;
            let records = object
                .records
                .iter()
                .zip(&input_object.records)
                .zip(&prelude.records)
                .map(|((record, input), flow)| match (record, input, flow) {
                    (Some(record), Some(input), Some(flow)) => cache::build(
                        &record.slices,
                        &record.fill_expolygons,
                        input_object
                            .region_options(input)
                            .ensure_vertical_shell_thickness,
                        flow.solid_infill_spacing,
                    )
                    .map(Some),
                    (None, None, None) => Ok(None),
                    _ => unreachable!("validated O19 slots remain aligned"),
                })
                .collect::<Result<_, _>>()?;
            Ok(VerticalShellCacheObject { records })
        })
        .collect()
}

fn validate_alignment(prepared: &PreparedPostFillSurfacePreparation) {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        assert_eq!(object.records.len(), input_object.records.len());
        assert_eq!(object.records.len(), prelude.records.len());
        let identity = input_object.identity();
        for ((record, input), flow) in object
            .records
            .iter()
            .zip(&input_object.records)
            .zip(&prelude.records)
        {
            match (record, input, flow) {
                (Some(_), Some(input), Some(_)) => {
                    assert_eq!((input.source_object_index, input.transform_index), identity);
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                }
                (None, None, None) => {}
                _ => panic!("O19 slots remain aligned with O18 and the Classic prelude"),
            }
        }
    }
}
