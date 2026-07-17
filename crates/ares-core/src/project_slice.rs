use crate::{
    GenerationMetadata, Project, SliceError,
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

mod bounds;
mod capabilities;
mod extruders;
mod layers;
mod parameters;
mod profile;
mod raw_intersections;
mod state;

#[cfg(test)]
mod tests;

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let state::ProjectSliceState {
        project,
        resolved,
        config_block,
        intersected_objects,
    } = state::prepare_project_slice(project)?;

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
    let BoundedResolvedProjectConfig {
        views,
        logical_filament_count,
        usage,
        print_object_count,
        objects,
    } = resolved;
    let full = views.full;
    let runtime = views.runtime;
    let runtime_gcode = views.runtime_gcode;
    let supported_used_filaments = usage.supported_used_filaments;
    let coverage = usage.coverage;
    for ResolvedProjectObject {
        source_object_index,
        object,
        print_objects,
        layer_candidates,
    } in objects
    {
        let _ = (source_object_index, object);
        for print_object in print_objects {
            let _ = print_object.transform;
        }
        for layer_candidate in layer_candidates {
            let _ = (
                layer_candidate.min_z,
                layer_candidate.max_z,
                layer_candidate.source_range_index,
            );
            for model_part in layer_candidate.model_parts {
                let _ = (model_part.volume_index, model_part.region);
            }
        }
    }
    for intersected_object in intersected_objects {
        let (plan, volumes) = intersected_object.into_parts();
        for volume in volumes {
            let (ordinal, volume_type, layers) = volume.into_parts();
            let _ = (ordinal, volume_type);
            for line in layers.into_iter().flatten() {
                let _ = (line.a(), line.b(), line.edge_type());
            }
        }
        let layers::PlannedPrintObject {
            source_object_index,
            transform_index,
            layers,
        } = plan;
        let _ = (source_object_index, transform_index);
        for layers::PlannedLayer {
            id,
            height,
            print_z,
            slice_z,
        } in layers
        {
            let _ = (id, height, print_z, slice_z);
        }
    }
    let _ = (
        project,
        full,
        runtime,
        runtime_gcode,
        logical_filament_count,
        supported_used_filaments,
        coverage,
        print_object_count,
        metadata,
        config_block,
    );
    Err(SliceError::ProjectSlicingIncomplete)
}

fn plan_project(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<Vec<layers::PlannedPrintObject>, SliceError> {
    capabilities::validate(
        project.has_painted_layer_height_profile(),
        project.objects(),
        &resolved.objects,
    )?;
    let object_extruders = extruders::collect_project_object_extruders(
        project.objects(),
        &resolved.objects,
        resolved.logical_filament_count,
    );
    plan_resolved_objects(&resolved.objects, |object_index, resolved_object| {
        let object_height = bounds::participating_object_heights(
            project.objects(),
            std::slice::from_ref(resolved_object),
        )?[0];
        parameters::slicing_parameters(
            &resolved.views.full,
            &resolved_object.object,
            object_height,
            &object_extruders[object_index],
        )
    })
}

fn plan_resolved_objects(
    resolved_objects: &[ResolvedProjectObject],
    mut prepare: impl FnMut(
        usize,
        &ResolvedProjectObject,
    ) -> Result<parameters::SlicingParameters, SliceError>,
) -> Result<Vec<layers::PlannedPrintObject>, SliceError> {
    let mut budget = layers::LayerBudget::default();
    let mut planned_objects = Vec::new();
    for (object_index, resolved_object) in resolved_objects.iter().enumerate() {
        if resolved_object.print_objects.is_empty() {
            continue;
        }
        let parameters = prepare(object_index, resolved_object)?;
        let profile = profile::fixed_layer_height_profile(&parameters);
        for (transform_index, _) in resolved_object.print_objects.iter().enumerate() {
            planned_objects.push(layers::plan_print_object(
                resolved_object.source_object_index,
                transform_index,
                &parameters,
                &profile,
                &mut budget,
            )?);
        }
        let parameters::SlicingParameters {
            base_raft_layers,
            interface_raft_layers,
            base_raft_layer_height,
            interface_raft_layer_height,
            contact_raft_layer_height,
            layer_height,
            min_layer_height,
            max_layer_height,
            first_print_layer_height,
            first_object_layer_height,
            first_object_layer_bridging,
            gap_raft_object,
            gap_object_support,
            gap_support_object,
            raft_base_top_z,
            raft_interface_top_z,
            raft_contact_top_z,
            object_print_z_min,
            object_print_z_max,
            object_print_z_uncompensated_max,
            object_shrinkage_compensation_z,
        } = parameters;
        let _ = (
            base_raft_layers,
            interface_raft_layers,
            base_raft_layer_height,
            interface_raft_layer_height,
            contact_raft_layer_height,
            layer_height,
            min_layer_height,
            max_layer_height,
            first_print_layer_height,
            first_object_layer_height,
            first_object_layer_bridging,
            gap_raft_object,
            gap_object_support,
            gap_support_object,
            raft_base_top_z,
            raft_interface_top_z,
            raft_contact_top_z,
            object_print_z_min,
            object_print_z_max,
            object_print_z_uncompensated_max,
            object_shrinkage_compensation_z,
        );
    }
    Ok(planned_objects)
}
