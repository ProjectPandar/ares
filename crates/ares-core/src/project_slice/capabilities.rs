use crate::{ProjectObject, SliceError, project::effective_config::types::ResolvedProjectObject};

pub(super) fn validate(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    spiral_mode: bool,
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
    if spiral_mode
        && resolved_objects.iter().any(|resolved| {
            source_objects[resolved.source_object_index]
                .layer_config_ranges()
                .iter()
                .any(|range| range.region_overrides() != &Default::default())
        })
    {
        return unsupported("layer_config_ranges");
    }
    if !spiral_mode
        && resolved_objects.iter().any(|object| {
            object.object.raft_layers.0 > 0
                && object.all_regions.iter().any(|region| {
                    region.seam_slope_type != crate::ProcessSeamScarfType::None
                        && region.seam_slope_conditional.0
                        && region.scarf_overhang_threshold.0 > 0.0
                })
        })
    {
        // Native UB: SeamPlacer.hpp:59-70 leaves unsupported_dist uninitialized
        // on object layer0 (:1092 assigns only with a previous layer; :1563 reads).
        // GCode.cpp:5787-5791 consumes it over a raft, whose print ID is >0.
        return unsupported("scarf_overhang_threshold");
    }
    Ok(())
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
