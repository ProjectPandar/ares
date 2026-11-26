use crate::{ProjectObject, SliceError, project::effective_config::types::ResolvedProjectObject};

pub(super) fn validate(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    if resolved_objects.iter().any(|resolved| {
        source_objects[resolved.source_object_index]
            .layer_height_profile()
            .is_empty()
            && source_objects[resolved.source_object_index]
                .layer_config_ranges()
                .iter()
                .any(|range| range.layer_height().is_some())
            && source_objects[resolved.source_object_index]
                .layer_config_ranges()
                .iter()
                .any(|range| range.layer_height().is_none())
    }) {
        // Slicing.cpp:249 is undefined without height; profile selection retains
        // this rejection if a nonempty stored profile must regenerate.
        return unsupported("layer_config_ranges");
    }
    Ok(())
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
