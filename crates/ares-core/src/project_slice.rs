use crate::{
    GenerationMetadata, Project, SliceError,
    geometry::CoordinateScale,
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

mod bounds;
mod capabilities;
mod chained_intersections;
mod closing;
mod extruders;
mod largest_contours;
mod layers;
mod looped_intersections;
mod parameters;
mod planning;
mod pre_closing_unions;
mod profile;
mod raw_intersections;
mod region_slices;
mod simplification;
mod slicing_mode_intersections;
mod state;
mod volume_bounds;
mod volume_regions;

#[cfg(any(test, feature = "task22j-browser-oracle"))]
mod task22g_oracle;
#[cfg(test)]
mod task22h_oracle;
#[cfg(any(test, feature = "task22j-browser-oracle"))]
mod task22i_oracle;
#[cfg(any(test, feature = "task22j-browser-oracle"))]
mod task22j_oracle;

#[cfg(test)]
mod tests;

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let PreparedPostRegions {
        project,
        resolved,
        config_block,
        scale,
        objects: post_region_objects,
    } = prepare_post_regions(project)?;

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
    for post_region_object in post_region_objects {
        let (plan, volume_slices, regions) = post_region_object.into_parts();
        let _ = (volume_slices, regions);
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
        scale,
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
    scale: CoordinateScale,
    objects: Vec<closing::PostClosingPrintObject>,
}

struct PreparedPostRegions {
    project: Project,
    resolved: BoundedResolvedProjectConfig,
    config_block: Option<Vec<u8>>,
    scale: CoordinateScale,
    objects: Vec<region_slices::PostRegionPrintObject>,
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
        scale,
        objects,
    })
}

fn prepare_post_largest_contours(
    project: impl AsRef<[u8]>,
) -> Result<PreparedPostClosing, SliceError> {
    let mut prepared = prepare_post_closing(project)?;
    largest_contours::apply_project_largest_contours(&mut prepared.objects);
    Ok(prepared)
}

fn prepare_post_simplification(
    project: impl AsRef<[u8]>,
) -> Result<PreparedPostClosing, SliceError> {
    let mut prepared = prepare_post_largest_contours(project)?;
    let resolution = prepared.resolved.views.full.process.print.resolution.0;
    simplification::apply_project_simplification(
        &mut prepared.objects,
        resolution,
        prepared.scale,
    )?;
    Ok(prepared)
}

fn prepare_post_regions(project: impl AsRef<[u8]>) -> Result<PreparedPostRegions, SliceError> {
    let PreparedPostClosing {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepare_post_simplification(project)?;
    let objects = {
        let mut contexts = resolved
            .objects
            .iter()
            .flat_map(|resolved| resolved.print_objects.iter().map(move |_| resolved));
        let objects = objects
            .into_iter()
            .map(|post_i| {
                let resolved_object = contexts
                    .next()
                    .expect("post-I object must have a resolved print-instance context");
                let source = &project.objects()[resolved_object.source_object_index];
                let bounded = volume_bounds::build_volume_bounds(source, resolved_object, post_i);
                let graph = volume_regions::build_volume_region_graph(
                    source,
                    resolved_object,
                    &bounded,
                    &resolved.views.full.filament.region,
                    resolved.logical_filament_count,
                );
                region_slices::complex::compose_complex_region_slices(
                    region_slices::prepare_region_slices(bounded, graph),
                    scale,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert!(
            contexts.next().is_none(),
            "every resolved print-instance context must have one post-I object"
        );
        objects
    };
    Ok(PreparedPostRegions {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

#[cfg(test)]
pub fn task22g_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_closing(project)?;
    Ok(task22g_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22h_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_closing(project)?;
    Ok(task22g_oracle::encode_with_magic(
        &prepared.objects,
        b"ARES22G\0",
    ))
}

#[cfg(test)]
pub fn task22h_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_largest_contours(project)?;
    Ok(task22h_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22i_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_largest_contours(project)?;
    Ok(task22h_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22i_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_simplification(project)?;
    Ok(task22i_oracle::encode(&prepared.objects))
}

#[cfg(any(test, feature = "task22j-browser-oracle"))]
pub fn task22j_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_simplification(project)?;
    Ok(task22i_oracle::encode(&prepared.objects))
}

#[cfg(any(test, feature = "task22j-browser-oracle"))]
pub fn task22j_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_regions(project)?;
    Ok(task22j_oracle::encode(&prepared.objects))
}
