use crate::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, Project, SliceError,
    geometry::CoordinateScale, project::effective_config::types::BoundedResolvedProjectConfig,
};

#[cfg(all(test, windows))]
pub(in crate::project_slice) const CONSTRAINED_TEST_STACK_SIZE: usize = 256 * 1024;

mod bounds;
mod capabilities;
mod closing;
pub(crate) mod compatibility;
mod compensation;
mod conical_overhang;
mod contour_z;
mod curl_estimation;
mod elephant_foot;
mod extruders;
mod extrusion_islands;
mod fill_entities;
pub(crate) mod gcode_emit;
mod group_fills;
mod hole_to_polyhole;
mod incomplete_sink;
mod island_print_order;
mod largest_contours;
mod layers;
mod load_stages;
mod looped_intersections;
mod parameters;
mod path_simplification;
mod raft;
pub(crate) use perimeters::classic::shortest_path::chain_polylines;
mod perimeters;
pub(crate) use perimeters::flow::skirt_flow_width_spacing;
mod planning;
mod print_height;
pub(crate) use print_height::validate as validate_print_height;
mod pre_closing_unions;
mod prepare_infill;
mod pressure_equalizer;
mod profile;
mod profile_entry;
mod validated;
pub(crate) use profile_entry::validate_profile_source;
pub(crate) use profile_entry::{center_profile_project, load_and_place};

pub use source::{CliUpwardMachine, SlicedPlate};
pub(crate) use validated::validate_post_process;
pub use validated::{
    CliProjectSliceRequest, slice_cli_project_plate_validated,
    slice_project_plate_processed_validated, slice_project_plate_validated,
};

/// Loads the project for a selected-profile request without slicing it —
/// the same load path `prepare_profile_slice` uses, exposed for the
/// `profile_settings` view.
pub(crate) fn load_profile_project(source: &ProjectSource<'_>) -> Result<Project, SliceError> {
    if let Some(refresh) = source.refresh {
        return source
            .input
            .load(source.overrides, Some(refresh), source.validate_print)
            .map(|(project, _)| project);
    }
    let Some(profile) = source.profile else {
        return Err(SliceError::InvalidInput(
            "a selected profile is required".to_owned(),
        ));
    };
    profile_entry::load_with_profile(source, profile)
}
mod centering;
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
mod source;
mod state;
use source::ProjectBytes;
pub(crate) use source::{ProfileInput, ProjectInput, ProjectSource};
pub(crate) use state::{BedShape, Circle, PrintVolume, instance_partly_outside};
mod support_material;

mod top_empty_layers;
mod volume_bounds;
mod volume_regions;

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    context: crate::SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    slice_project_plate(
        project,
        metadata,
        None,
        &ConfigOverrides::default(),
        context,
    )
    .await
}

/// Slices one plate of a multi-plate 3MF project. `plate` selects the plate
// by its `plater_id`; `None` selects the first plate.
pub async fn slice_project_plate(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: crate::SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    slice_source(
        ProjectSource {
            input: crate::project_slice::ProjectInput::Bytes(project.as_ref()),
            plate,
            profile: None,
            refresh: None,
            overrides,
            validate_print: false,
            validate_config: true,
            orient: false,
            upward: None,
            sliced_plate: None,
            edits: &crate::plater::NO_EDITS,
        },
        metadata,
        false,
        context,
    )
    .map(|processed| processed.gcode)
}

/// Slices one plate and also returns the `GCodeProcessorResult` move
/// vertices the processor pass records over the exported G-code.
pub async fn slice_project_plate_processed(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: crate::SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_source(
        ProjectSource {
            input: crate::project_slice::ProjectInput::Bytes(project.as_ref()),
            plate,
            profile: None,
            refresh: None,
            overrides,
            validate_print: false,
            validate_config: true,
            orient: false,
            upward: None,
            sliced_plate: None,
            edits: &crate::plater::NO_EDITS,
        },
        metadata,
        true,
        context,
    )
}

#[inline(never)]
pub(crate) fn slice_source(
    mut project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    mut context: crate::SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    // Fresh-process equivalence for the `~`/`^` rotate_template joints:
    // upstream slices once per process, so `rand()` always sees the default
    // seed (`Fill.cpp:199-200`); reset the per-thread stream per slice.
    group_fills::reset_rand_state();
    let mut sliced_plate = project.sliced_plate.take();
    let project = ProjectSource {
        sliced_plate: sliced_plate.as_deref_mut(),
        ..project
    };
    let walls = perimeters::prepare_post_layer_region_perimeters(project, &mut context)?;
    // `Print.cpp:3461-3481` prepares and fills each object in turn (`PrintObject.cpp:631,819`);
    // Ares prepares the infill of every object, then fills every object.
    for _ in &walls.objects {
        context.set_status(25, format_args!("Generating infill regions"), 0);
        context.set_status(35, format_args!("Generating infill toolpath"), 0);
    }
    let staged = prepare_infill::surface_type_detection::prepare(walls)?;
    let external = prepare_infill::external_surfaces::prepare(
        prepare_infill::horizontal_shell_propagation::prepare(
            prepare_infill::horizontal_shell_promotion::prepare(
                prepare_infill::vertical_shell_assignment::prepare(
                    prepare_infill::vertical_shell_filtering::prepare(
                        prepare_infill::vertical_shell_regularization::prepare(
                            prepare_infill::vertical_shell_trimming::prepare(
                                prepare_infill::vertical_shell_projection::prepare(
                                    prepare_infill::vertical_shells::prepare(
                                        prepare_infill::fill_surfaces::prepare(staged),
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
    // `get_boundary` consumes current Top fill surfaces after prepare_infill
    // (`AvoidCrossingPerimeters.cpp:1116-1130`), not the classification snapshot.
    let top_surfaces = extract_top_surfaces(&prepared.predecessor.predecessor.predecessor);
    let mut filled = fill_entities::prepare(prepared)?;
    contour_z::prepare(&mut filled, &mut context);
    let mut warnings = match support_material::check_necessity(&filled, &mut context) {
        Ok(warnings) => warnings,
        Err(error) => {
            fill_entities::dispose(filled);
            return Err(error);
        }
    };
    if let Some(sliced_plate) = sliced_plate.as_deref_mut() {
        // CLI:7150-7160 records warnings before export; a later error keeps them.
        sliced_plate.warnings.clone_from(&warnings);
    }
    let support_layers = match support_material::generate(&filled, &mut context) {
        Ok(layers) => layers,
        Err(error) => {
            fill_entities::dispose(filled);
            return Err(error);
        }
    };
    let islands = extrusion_islands::prepare(filled);
    let mut ordered = island_print_order::prepare(islands, top_surfaces);
    ordered.support_layers = support_layers;
    ordered.curled_lines = curl_estimation::estimate(&ordered)?;
    path_simplification::apply(&mut ordered);
    seam_placement::apply(&mut ordered);
    let output = consume_post_island_print_order(
        ordered,
        metadata,
        record_moves,
        &mut context,
        &mut warnings,
    );
    if let Some(sliced_plate) = sliced_plate {
        sliced_plate.warnings.clone_from(&warnings);
    }
    let mut output = output?;
    output.warnings = warnings;
    Ok(output)
}

fn extract_top_surfaces(
    staged: &prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
) -> Vec<Vec<Vec<crate::geometry::ExPolygon>>> {
    staged
        .objects
        .iter()
        .zip(&staged.predecessor.objects)
        .map(|(object, traversal)| {
            let layer_count = traversal
                .records
                .iter()
                .flatten()
                .map(|record| record.layer_id + 1)
                .max()
                .unwrap_or(0);
            let mut layers = vec![Vec::new(); layer_count];
            for (record, input) in object.records.iter().zip(&traversal.records) {
                let (Some(record), Some(input)) = (record, input) else {
                    continue;
                };
                let layer = &mut layers[input.layer_id];
                layer.extend(
                    record
                        .fill_surfaces
                        .iter()
                        .filter(|surface| {
                            matches!(
                                surface.as_parts().0,
                                crate::project_slice::region_slices::RegionSurfaceKind::Top,
                            )
                        })
                        .map(|surface| surface.as_parts().1.clone()),
                );
            }
            layers
        })
        .collect()
}

#[inline(never)]
fn consume_post_island_print_order(
    mut prepared: island_print_order::PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: &mut crate::SlicerContext,
    warnings: &mut Vec<crate::PrintWarning>,
) -> Result<ProcessedGCode, SliceError> {
    // `Print.cpp:3540-3550` detects auto-lift overhangs per object (`PrintObject.cpp:940`; Ares runs
    // the detection in `AutoLift::prepare`), then makes the skirt and brim (`Print.cpp:3638`).
    for _ in &prepared.objects {
        context.set_status(71, format_args!("Detect overhangs for auto-lift"), 0);
    }
    context.set_status(70, format_args!("Generating skirt & brim"), 0);
    let mut output = gcode_emit::emit(&mut prepared, metadata, record_moves, context, warnings)?;
    output.result.apply_filament_config(prepared.filament());
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
    painting: Vec<region_slices::painted::ObjectRegions>,
}
