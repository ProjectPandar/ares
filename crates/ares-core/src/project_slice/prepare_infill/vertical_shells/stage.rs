use crate::{
    CancelToken, SliceError, parallel::par_range_map_ordered_try,
    project_slice::prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
};

use super::{cache, types::VerticalShellCacheObject};

pub(super) fn project(
    prepared: &PreparedPostFillSurfacePreparation,
    cancel: &CancelToken,
) -> Result<Vec<VerticalShellCacheObject>, SliceError> {
    cancel.check()?;
    validate_alignment(prepared);
    (0..prepared.objects.len())
        .map(|object_index| project_object(prepared, object_index, cancel))
        .collect()
}

fn project_object(
    prepared: &PreparedPostFillSurfacePreparation,
    object_index: usize,
    cancel: &CancelToken,
) -> Result<VerticalShellCacheObject, SliceError> {
    let object = &prepared.objects[object_index];
    let traversal = &prepared.predecessor.objects[object_index];
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input_object = &prelude.object;
    let (compensated, _) = input_object.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    let resolved = &prepared.predecessor.resolved;
    let options = &resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == plan.source_object_index)
        .expect("vertical shells retain their resolved object")
        .object;
    let num_layers = if resolved.views.full.process.print.spiral_mode.0 {
        plan.layers
            .len()
            .min(regions[0].options.bottom_shell_layers.0 as usize)
    } else {
        plan.layers.len()
    };
    let multiple = regions.len() > 1;
    let shared = multiple && !options.interface_shells.0;
    let records = par_range_map_ordered_try(object.records.len(), cancel, |index| {
        match (&object.records[index], &input_object.records[index]) {
            (Some(_), Some(input)) if shared || input.current.layer_index >= num_layers => {
                Ok(Some(cache::empty()))
            }
            (Some(record), Some(input)) => {
                let fill_expolygons = if multiple {
                    &[][..]
                } else {
                    record.fill_expolygons.as_slice()
                };
                cache::build(
                    &record.slices,
                    fill_expolygons,
                    input_object
                        .region_options(input)
                        .ensure_vertical_shell_thickness,
                    input
                        .solid_infill_flow
                        .scaled_spacing(prepared.predecessor.scale),
                )
                .map(Some)
            }
            (None, None) => Ok(None),
            _ => unreachable!("validated O19 slots remain aligned"),
        }
    })?;
    let physical = if multiple
        && regions.iter().any(|region| {
            region.options.ensure_vertical_shell_thickness
                == crate::ProcessEnsureVerticalShellThickness::EnsureAll
        }) {
        Some(super::physical_cache::build(
            prepared,
            object_index,
            num_layers,
            shared,
            cancel,
        )?)
    } else {
        None
    };
    Ok(VerticalShellCacheObject {
        records,
        physical,
        num_layers,
    })
}

fn validate_alignment(prepared: &PreparedPostFillSurfacePreparation) {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let input_object = &prelude.object;
        assert_eq!(object.records.len(), input_object.records.len());
        let identity = input_object.identity();
        for (record, input) in object.records.iter().zip(&input_object.records) {
            match (record, input) {
                (Some(_), Some(input)) => {
                    assert_eq!((input.source_object_index, input.transform_index), identity);
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                }
                (None, None) => {}
                _ => panic!("O19 slots remain aligned with O18 and regional inputs"),
            }
        }
    }
}
