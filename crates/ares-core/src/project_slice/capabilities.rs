use crate::{
    ProjectObject, ProjectVolumeType, SliceError,
    project::effective_config::types::ResolvedProjectObject,
};

pub(super) fn validate(
    has_painted_layer_height_profile: bool,
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    _spiral_mode: bool,
) -> Result<(), SliceError> {
    if has_painted_layer_height_profile {
        return unsupported("layer_height_profile");
    }
    if resolved_objects.iter().any(|resolved| {
        source_objects[resolved.source_object_index]
            .layer_config_ranges()
            .iter()
            .any(|range| range.layer_height().is_some())
    }) {
        return unsupported("layer_height");
    }
    if resolved_objects
        .iter()
        .any(|resolved| resolved.object.raft_layers.0 != 0)
    {
        return unsupported("raft_layers");
    }
    if resolved_objects.iter().any(|resolved| {
        resolved
            .layer_candidates
            .iter()
            .flat_map(|candidate| &candidate.model_parts)
            .any(|part| part.region.zaa_enabled.0)
            || source_objects[resolved.source_object_index]
                .volumes()
                .iter()
                .filter(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
                .any(|volume| {
                    volume
                        .region_overrides()
                        .zaa_enabled
                        .is_some_and(|value| value.0)
                })
    }) {
        return unsupported("zaa_enabled");
    }
    Ok(())
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
