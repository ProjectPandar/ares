use crate::{Project, project::effective_config::types::BoundedResolvedProjectConfig};

pub(super) fn observe(project: &Project, resolved: &BoundedResolvedProjectConfig) {
    let documents = project.documents();
    let _ = (
        project.models(),
        project.objects(),
        project.plates(),
        project.settings(),
        project.has_painted_layer_height_profile(),
        &documents.model_settings,
        &documents.slice_info,
        &documents.filament_sequences,
        &documents.plate_documents,
        documents.has_painted_layer_height_profile,
    );
    let _ = (
        &resolved.views.full,
        &resolved.views.runtime,
        &resolved.views.runtime_gcode,
        resolved.logical_filament_count,
        &resolved.usage.supported_used_filaments,
        &resolved.usage.coverage,
        resolved.print_object_count,
    );
    for object in &resolved.objects {
        let _ = (object.source_object_index, &object.object);
        for print_object in &object.print_objects {
            let _ = print_object.transform;
        }
        for layer_candidate in &object.layer_candidates {
            let _ = (
                layer_candidate.min_z,
                layer_candidate.max_z,
                layer_candidate.source_range_index,
            );
            for model_part in &layer_candidate.model_parts {
                let _ = (model_part.volume_index, &model_part.region);
            }
        }
    }
}
