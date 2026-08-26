use crate::{
    GenerationMetadata, Project, SliceError, geometry::CoordinateScale,
    project::effective_config::types::BoundedResolvedProjectConfig,
};

#[cfg(all(test, windows))]
pub(in crate::project_slice) const CONSTRAINED_TEST_STACK_SIZE: usize = 256 * 1024;
#[cfg(all(test, not(windows)))]
pub(in crate::project_slice) const CONSTRAINED_TEST_STACK_SIZE: usize = 64 * 1024;

mod bounds;
mod capabilities;
mod chained_intersections;
mod closing;
mod compensation;
mod conical_overhang;
mod elephant_foot;
mod extruders;
mod extrusion_islands;
mod fill_entities;
mod gcode_emit;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "full fill grouping remains inactive until a later source-cited lifecycle slice"
    )
)]
mod group_fills;
mod incomplete_sink;
mod island_print_order;
mod largest_contours;
mod layers;
mod looped_intersections;
mod parameters;
mod path_simplification;
mod perimeters;
mod planning;
mod pre_closing_unions;
mod prepare_infill;
mod profile;
mod raw_intersections;
mod region_slices;
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "pure O97 topology activates with seam placement")
)]
mod seam_candidates;
mod seam_placement;
mod simplification;
mod slice_ordering;
mod slicing_mode_intersections;
mod state;
#[cfg(test)]
mod test_consumers;
mod top_empty_layers;
mod volume_bounds;
mod volume_regions;
#[cfg(test)]
use test_consumers::*;

#[cfg(test)]
mod tests;

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    slice_project_sync(project, metadata)
}

#[inline(never)]
fn slice_project_sync(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let external = prepare_infill::external_surfaces::prepare(
        prepare_infill::horizontal_shell_propagation::prepare(
            prepare_infill::horizontal_shell_promotion::prepare(
                prepare_infill::vertical_shell_assignment::prepare(
                    prepare_infill::vertical_shell_filtering::prepare(
                        prepare_infill::vertical_shell_regularization::prepare(
                            prepare_infill::vertical_shell_trimming::prepare(
                                prepare_infill::vertical_shell_projection::prepare(
                                    prepare_infill::vertical_shells::prepare(
                                        prepare_infill::fill_surfaces::prepare(
                                            prepare_infill::surface_type_detection::prepare(
                                                perimeters::prepare_post_layer_region_perimeters(
                                                    project,
                                                )?,
                                            )?,
                                        ),
                                    )?,
                                )?,
                            )?,
                        )?,
                    )?,
                )?,
            )?,
        )?,
    )?;
    let candidates = prepare_infill::bridge_over_infill::prepare(external)?;
    let bridged = prepare_infill::bridge_over_infill::transaction::prepare(candidates)?;
    let prepared = prepare_infill::combine_infill::prepare(bridged)?;
    let filled = fill_entities::prepare(prepared)?;
    let islands = extrusion_islands::prepare(filled);
    let mut ordered = island_print_order::prepare(islands);
    path_simplification::apply(&mut ordered);
    seam_placement::apply(&mut ordered);
    consume_post_island_print_order(ordered, metadata)
}

#[inline(never)]
fn consume_post_island_print_order(
    mut prepared: island_print_order::PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let output = gcode_emit::emit(&mut prepared, metadata)?;
    island_print_order::dispose(prepared);
    Ok(output)
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

fn prepare_post_top_empty_layers(
    project: impl AsRef<[u8]>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_regions(project)?;
    top_empty_layers::remove_project_top_empty_layers(&mut prepared.objects);
    Ok(prepared)
}

fn prepare_post_conical_overhang(
    project: impl AsRef<[u8]>,
) -> Result<PreparedPostRegions, SliceError> {
    let mut prepared = prepare_post_top_empty_layers(project)?;
    conical_overhang::apply_project_conical_overhang(
        &mut prepared.objects,
        &prepared.resolved.objects,
        prepared.scale,
    )?;
    Ok(prepared)
}
