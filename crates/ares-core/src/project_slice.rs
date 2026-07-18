use crate::{
    GenerationMetadata, Project, SliceError,
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

mod bounds;
mod capabilities;
mod chained_intersections;
mod closing;
mod extruders;
mod layers;
mod looped_intersections;
mod parameters;
mod pre_closing_unions;
mod profile;
mod raw_intersections;
mod slicing_mode_intersections;
mod state;

#[cfg(any(test, feature = "task22g-browser-oracle"))]
mod task22g_oracle;

#[cfg(test)]
mod tests;

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let PreparedPostClosing {
        project,
        resolved,
        config_block,
        objects: post_closing_objects,
    } = prepare_post_closing(project)?;

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
    for post_closing_object in post_closing_objects {
        let (plan, volumes) = post_closing_object.into_parts();
        for (mode, expolygons) in volumes
            .into_iter()
            .flat_map(|volume| {
                let (source_volume_index, ordinal, volume_type, layers) = volume.into_parts();
                let _ = (source_volume_index, ordinal, volume_type);
                layers
            })
            .map(closing::PostClosingLayer::into_parts)
        {
            let _ = mode;
            for polygon in expolygons.into_iter().flat_map(|expolygon| {
                let (contour, holes) = expolygon.into_parts();
                std::iter::once(contour).chain(holes)
            }) {
                let _ = polygon.points();
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

struct PreparedPostClosing {
    project: Project,
    resolved: BoundedResolvedProjectConfig,
    config_block: Option<Vec<u8>>,
    objects: Vec<closing::PostClosingPrintObject>,
}

fn prepare_post_closing(project: impl AsRef<[u8]>) -> Result<PreparedPostClosing, SliceError> {
    let state::ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    } = state::prepare_project_slice(project)?;
    let chained_objects = chained_intersections::chain_project_intersections(intersected_objects);
    let max_gap_scaled = scale
        .checked_scale(2.0)
        .expect("2 mm loop-repair radius must fit the selected coordinate scale");
    let looped_objects =
        looped_intersections::loop_project_intersections(chained_objects, max_gap_scaled);
    let spiral_mode = resolved.views.full.process.print.spiral_mode.0;
    let slicing_mode_objects = slicing_mode_intersections::apply_project_slicing_modes(
        looped_objects,
        &resolved.objects,
        spiral_mode,
    )?;
    let pre_closing_objects =
        pre_closing_unions::apply_project_pre_closing_unions(slicing_mode_objects)?;
    let objects = closing::apply_project_closing(pre_closing_objects, &resolved.objects, scale)?;
    Ok(PreparedPostClosing {
        project,
        resolved,
        config_block,
        objects,
    })
}

#[cfg(any(test, feature = "task22g-browser-oracle"))]
pub fn task22g_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_closing(project)?;
    Ok(task22g_oracle::encode(&prepared.objects))
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
