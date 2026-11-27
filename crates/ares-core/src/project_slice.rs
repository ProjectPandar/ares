use crate::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, Project, SliceError,
    geometry::CoordinateScale, project::effective_config::types::BoundedResolvedProjectConfig,
};

#[cfg(all(test, windows))]
pub(in crate::project_slice) const CONSTRAINED_TEST_STACK_SIZE: usize = 256 * 1024;

mod archive_plate;
mod belt;
pub(crate) use belt::kinematics::MachineFrameTransform;
mod bounds;
mod capabilities;
mod closing;
pub(crate) mod compatibility;
mod compensation;
pub(crate) use archive_plate::{
    ArchivePlate, ArchiveSource, slice_source, slice_source_to_sink, slice_source_with_archive,
    slice_source_with_custom_gcodes,
};
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
mod island_print_order;
mod largest_contours;
mod layers;
mod load_stages;
mod looped_intersections;
mod parameters;
mod path_simplification;
mod raft;
pub(crate) use perimeters::classic::shortest_path::chain_polylines;
pub(crate) mod option_stl;
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
mod cli_plates;
mod cli_stl;
mod streaming;
pub use cli_plates::{
    CliPlateFailure, CliPlateFailureKind, CliPlatesRequest, CliPlatesSettings,
    slice_cli_project_plates, slice_cli_project_plates_to_sink,
};
pub use cli_stl::{SlicedStlPlate, StlPlateFailure, slice_stl_models_with_load_settings};
pub use streaming::{
    slice_cli_project_plate_validated_to_sink,
    slice_cli_project_plate_validated_with_catalog_to_sink, slice_project_plate_processed_to_sink,
    slice_project_plate_to_sink, slice_project_plate_validated_to_sink,
};
pub use validated::{
    CliProjectSliceRequest, DirectProjectSliceRequest, slice_cli_project_plate_validated,
    slice_cli_project_plate_validated_with_catalog, slice_cli_project_plate_with_refresh,
    slice_project_plate_processed_validated, slice_project_plate_processed_validated_with_catalog,
    slice_project_plate_processed_validated_with_custom_gcodes, slice_project_plate_validated,
    slice_project_plate_validated_with_catalog, slice_project_plate_with_archive,
};
pub(crate) use validated::{slice_loaded_project_plate_with_archive, validate_post_process};

/// Loads the project for a selected-profile request without slicing it —
/// the same load path `prepare_profile_slice` uses, exposed for the
/// `profile_settings` view.
pub(crate) fn load_profile_project(source: &ProjectSource<'_>) -> Result<Project, SliceError> {
    if let Some(refresh) = source.refresh {
        return source
            .input
            .load(
                source.overrides,
                Some(refresh),
                source.system_catalog,
                (source.validate_print, source.skip_modified_gcodes, None),
            )
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
    slice_project_plate_with_catalog(project, metadata, (plate, overrides), None, context).await
}

pub async fn slice_project_plate_with_catalog(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    (plate, overrides): (Option<u32>, &ConfigOverrides),
    system_catalog: Option<&crate::PresetCatalogHandle>,
    context: crate::SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    let mut gcode = Vec::new();
    streaming::slice_project_plate_to_sink_inner(
        project.as_ref(),
        DirectProjectSliceRequest {
            plate,
            overrides,
            system_catalog,
        },
        (metadata, false, false),
        context,
        &mut gcode,
    )?;
    Ok(gcode)
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
    slice_project_plate_processed_with_catalog(project, metadata, (plate, overrides), None, context)
        .await
}

pub async fn slice_project_plate_processed_with_catalog(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    (plate, overrides): (Option<u32>, &ConfigOverrides),
    system_catalog: Option<&crate::PresetCatalogHandle>,
    context: crate::SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    let mut gcode = Vec::new();
    let streamed = streaming::slice_project_plate_to_sink_inner(
        project.as_ref(),
        DirectProjectSliceRequest {
            plate,
            overrides,
            system_catalog,
        },
        (metadata, true, false),
        context,
        &mut gcode,
    )?;
    Ok(streamed.into_processed(gcode))
}

fn slice_plate(
    generation: (GenerationMetadata, bool),
    context: &mut crate::SlicerContext<'_>,
    outputs: (
        Option<&mut SlicedPlate>,
        Option<&mut Vec<crate::gcode_processor::CustomGCodeItem>>,
    ),
    archive: Option<ArchiveSource>,
    prepare: impl FnOnce(
        Option<&mut SlicedPlate>,
        &mut crate::SlicerContext<'_>,
    ) -> Result<
        perimeters::layer_region::PreparedPostLayerRegionPerimeters,
        SliceError,
    >,
) -> Result<ProcessedGCode, SliceError> {
    let mut gcode = Vec::new();
    let streamed =
        slice_plate_to_sink(generation, (context, &mut gcode), outputs, archive, prepare)?;
    Ok(streamed.into_processed(gcode))
}

fn slice_plate_to_sink(
    generation: (GenerationMetadata, bool),
    (context, sink): (&mut crate::SlicerContext<'_>, &mut dyn crate::GCodeSink),
    (mut sliced_plate, plate_custom_gcodes): (
        Option<&mut SlicedPlate>,
        Option<&mut Vec<crate::gcode_processor::CustomGCodeItem>>,
    ),
    archive: Option<ArchiveSource>,
    prepare: impl FnOnce(
        Option<&mut SlicedPlate>,
        &mut crate::SlicerContext<'_>,
    ) -> Result<
        perimeters::layer_region::PreparedPostLayerRegionPerimeters,
        SliceError,
    >,
) -> Result<crate::StreamedGCode, SliceError> {
    // Fresh-process equivalence for the `~`/`^` rotate_template joints
    // (`Fill.cpp:199-200`).
    group_fills::reset_rand_state();
    context.check_canceled()?;
    let walls = prepare(sliced_plate.as_deref_mut(), context)?;
    context.check_canceled()?;
    // `Print.cpp:3461-3481` prepares and fills each object in turn (`PrintObject.cpp:631,819`);
    // Ares prepares the infill of every object, then fills every object.
    for _ in &walls.objects {
        context.set_status(25, format_args!("Generating infill regions"), 0);
        context.check_canceled()?;
        context.set_status(35, format_args!("Generating infill toolpath"), 0);
        context.check_canceled()?;
    }
    let prepared = prepare_infill::prepare(walls, context.cancel_ref())?;
    context.check_canceled()?;
    // `get_boundary` consumes current Top fill surfaces after prepare_infill
    // (`AvoidCrossingPerimeters.cpp:1116-1130`), not the classification snapshot.
    let top_surfaces = extract_top_surfaces(&prepared.predecessor.predecessor.predecessor);
    let mut filled = fill_entities::prepare(prepared, context.cancel_ref())?;
    context.check_canceled()?;
    contour_z::prepare(&mut filled, context)?;
    context.check_canceled()?;
    let mut warnings = support_material::check_necessity(&filled, context)?;
    context.check_canceled()?;
    if let Some(sliced_plate) = sliced_plate.as_deref_mut() {
        // CLI:7150-7160 records warnings before export; a later error keeps them.
        sliced_plate.warnings.clone_from(&warnings);
    }
    let support_layers = support_material::generate(&filled, context)?;
    context.check_canceled()?;
    let islands = extrusion_islands::prepare(filled);
    let mut ordered = island_print_order::prepare(islands, top_surfaces);
    ordered.support_layers = support_layers;
    context.check_canceled()?;
    ordered.curled_lines = curl_estimation::estimate(&ordered)?;
    path_simplification::apply(&mut ordered);
    context.check_canceled()?;
    if let Some(custom_gcodes) = plate_custom_gcodes {
        *custom_gcodes = ordered.plate_custom_gcodes();
    }
    let output = consume_post_island_print_order(
        ordered,
        generation,
        (context, sink),
        &mut warnings,
        archive,
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
    generation: (GenerationMetadata, bool),
    (context, sink): (&mut crate::SlicerContext, &mut dyn crate::GCodeSink),
    warnings: &mut Vec<crate::PrintWarning>,
    archive: Option<ArchiveSource>,
) -> Result<crate::StreamedGCode, SliceError> {
    let mut output = gcode_emit::emit_to_sink(
        &mut prepared,
        generation,
        (context, sink),
        warnings,
        archive,
    )?;
    output.result.apply_filament_config(prepared.filament());
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
