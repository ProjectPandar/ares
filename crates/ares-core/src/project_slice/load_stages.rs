use crate::SliceError;

use super::{
    PreparedPostClosing, PreparedPostRegions, ProjectSource, chained_intersections, closing,
    conical_overhang, largest_contours, looped_intersections, pre_closing_unions, profile_entry,
    region_slices, simplification, slicing_mode_intersections, state, top_empty_layers,
    volume_bounds, volume_regions,
};

pub(super) fn prepare_post_closing(
    mut project: ProjectSource<'_>,
) -> Result<PreparedPostClosing, SliceError> {
    let sliced_plate = project.sliced_plate.take();
    let state::ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    } = if let Some(profile) = project.profile {
        profile_entry::prepare_profile_slice(&project, profile)?
    } else {
        state::prepare_project_source(project)?
    };
    if let Some(sliced_plate) = sliced_plate {
        // `OrcaSlicer.cpp:6576-6640, 6926-6931`: the plate's facets and `m_print_config` values, read
        // as `Print::validate`'s failure families read them (`print/validate.rs`).
        let settings = &resolved.views.full;
        *sliced_plate = crate::SlicedPlate {
            warnings: Vec::new(),
            triangle_count: state::inside_triangle_count(&project)?,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        };
    }
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

pub(super) fn prepare_post_largest_contours(
    project: ProjectSource<'_>,
) -> Result<PreparedPostClosing, SliceError> {
    let mut prepared = prepare_post_closing(project)?;
    largest_contours::apply_project_largest_contours(&mut prepared.objects);
    Ok(prepared)
}

pub(super) fn prepare_post_simplification(
    project: ProjectSource<'_>,
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

pub(super) fn prepare_post_regions(
    project: ProjectSource<'_>,
) -> Result<PreparedPostRegions, SliceError> {
    let PreparedPostClosing {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepare_post_simplification(project)?;
    let mut painting = Vec::new();
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
                let mmu_painted = source.is_mm_painted(resolved.logical_filament_count);
                let bounded = if mmu_painted {
                    volume_bounds::build_volume_bounds_with_inflation(
                        source,
                        resolved_object,
                        post_i,
                        0.0,
                    )
                } else {
                    volume_bounds::build_volume_bounds(source, resolved_object, post_i)
                };
                let (graph, painted_regions) = volume_regions::build_volume_region_graph(
                    source,
                    resolved_object,
                    &bounded,
                    (
                        &resolved.views.full.filament.region,
                        resolved.logical_filament_count,
                        resolved.process_variant_indices.as_slice(),
                    ),
                );
                painting.push(region_slices::painted::ObjectRegions::new(
                    &graph,
                    painted_regions,
                    mmu_painted,
                ));
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
        painting,
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

pub(super) fn prepare_post_top_empty_layers(
    project: ProjectSource<'_>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_regions(project)?;
    top_empty_layers::remove_project_top_empty_layers(&mut prepared.objects);
    // PrintObjectSlice.cpp:866-867 rejects an empty sliced object before perimeters.
    if prepared
        .objects
        .iter()
        .any(|object| object.plan.layers.is_empty())
    {
        return Err(SliceError::InvalidInput("no layers were detected".into()));
    }
    Ok(prepared)
}

pub(super) fn prepare_post_conical_overhang(
    project: ProjectSource<'_>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_top_empty_layers(project)?;
    conical_overhang::apply_project_conical_overhang(
        &mut prepared.objects,
        &prepared.resolved.objects,
        prepared.scale,
    )?;
    Ok(prepared)
}

pub(super) fn prepare_post_painted(
    project: ProjectSource<'_>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_conical_overhang(project)?;
    region_slices::painted::apply_project(&mut prepared)?;
    region_slices::painted::apply_fuzzy_project(&mut prepared)?;
    Ok(prepared)
}

pub(super) fn prepare_post_interlocking(
    project: ProjectSource<'_>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_painted(project)?;
    let contexts = prepared
        .resolved
        .objects
        .iter()
        .flat_map(|resolved| resolved.print_objects.iter().map(move |_| resolved));
    // PrintObjectSlice.cpp:1243 consumes the post-MMU region surfaces before compensation.
    for (object, resolved) in prepared.objects.iter_mut().zip(contexts) {
        region_slices::interlocking::apply(
            object,
            &resolved.object,
            &prepared.resolved.views.full.project.print.nozzle_diameter,
            prepared.scale,
        )?;
    }
    Ok(prepared)
}
