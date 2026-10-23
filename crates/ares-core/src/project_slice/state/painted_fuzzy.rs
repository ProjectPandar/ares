//! Admission boundary for the bounded PrintApply/MMS/FuzzySkin painted rewrite.
use crate::{
    ProcessFuzzySkinMode, ProcessNoiseType, ProcessPerimeterGenerator, Project, ProjectVolumeType,
    SliceError, project::effective_config::types::BoundedResolvedProjectConfig,
};

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<(), SliceError> {
    if !project
        .objects()
        .iter()
        .any(|object| object.is_fuzzy_skin_painted())
    {
        return Ok(());
    }
    let full = &resolved.views.full;
    if resolved.objects.len() != 1
        || full.filament.gcode.filament_diameter.0.len() != 1
        || full.process.print.enable_prime_tower.0
        || !project.curr_plate_custom_gcodes().gcodes.is_empty()
    {
        return Err(deferred());
    }
    for object in &resolved.objects {
        let source = &project.objects()[object.source_object_index];
        let options = &object.object;
        if source.instances().len() != 1
            || source.volumes().len() != 1
            || source.volumes().iter().any(|volume| {
                volume.volume_type() != ProjectVolumeType::ModelPart
                    || !volume.mesh().mmu_segmentation_facets.mapping.is_empty()
            })
            || options.wall_generator != ProcessPerimeterGenerator::Classic
            || options.interlocking_beam.0
            || options.enable_support.0
            || options.enforce_support_layers.0 > 0
            || options.raft_layers.0 != 0
            || options.xy_contour_compensation.0 != 0.0
            || options.xy_hole_compensation.0 != 0.0
            || object
                .layer_candidates
                .iter()
                .flat_map(|range| &range.model_parts)
                .any(|part| {
                    part.region.fuzzy_skin_noise_type != ProcessNoiseType::Ripple
                        || part.region.fuzzy_skin_mode != ProcessFuzzySkinMode::Displacement
                })
        {
            return Err(deferred());
        }
    }
    Ok(())
}

fn deferred() -> SliceError {
    SliceError::InvalidInput(
        "painted fuzzy-skin composition is outside the bounded classic/Ripple domain".to_owned(),
    )
}
