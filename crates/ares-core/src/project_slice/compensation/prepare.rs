//! PrintObjectSlice.cpp:1250–1254 suppresses painted XY compensation, not elephant foot.

use super::{
    PostCompensationPrintObject, apply_object_compensation, prepare_object_compensation,
    validate_structures, validate_task22m_configs,
};
use crate::{
    FloatOrPercent, OrcaFloats, SliceError, geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::region_slices::PostRegionPrintObject,
};

pub(in crate::project_slice) fn apply_project_compensation(
    objects: Vec<PostRegionPrintObject>,
    resolved_objects: &[ResolvedProjectObject],
    initial_layer_width: FloatOrPercent,
    nozzle_diameters: &OrcaFloats,
    (scale, painting): (CoordinateScale, impl Iterator<Item = bool>),
) -> Result<Vec<PostCompensationPrintObject>, SliceError> {
    let contexts = resolved_objects
        .iter()
        .flat_map(|resolved| {
            resolved
                .print_objects
                .iter()
                .enumerate()
                .map(move |(transform_index, _)| (resolved, transform_index))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        objects.len(),
        contexts.len(),
        "Task 22M objects must match resolved print-object contexts"
    );
    for (object, (resolved, transform_index)) in objects.iter().zip(&contexts) {
        assert_eq!(
            object.plan.source_object_index,
            resolved.source_object_index
        );
        assert_eq!(object.plan.transform_index, *transform_index);
    }

    let options = contexts
        .iter()
        .map(|(resolved, _)| &resolved.object)
        .collect::<Vec<_>>();
    let mut configs = validate_task22m_configs(&options)?;
    for (config, painted) in configs.iter_mut().zip(painting) {
        if painted {
            config.xy_contour_mm = 0.0;
            config.xy_hole_mm = 0.0;
        }
    }
    validate_structures(&objects);
    let prepared = objects
        .iter()
        .zip(&configs)
        .map(|(object, config)| {
            prepare_object_compensation(
                object,
                *config,
                initial_layer_width,
                nozzle_diameters,
                scale,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut closing_epsilon = None;
    objects
        .into_iter()
        .zip(prepared)
        .map(|(object, prepared)| {
            apply_object_compensation(
                object,
                prepared,
                nozzle_diameters,
                scale,
                &mut closing_epsilon,
            )
        })
        .collect()
}
