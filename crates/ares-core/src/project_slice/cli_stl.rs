//! Native CLI's apply and process passes for arranged STL inputs.

use crate::{
    GenerationMetadata, ImportedStl, LoadSettingsSliceRequest, ProcessedGCode, Project, SliceError,
    SlicedPlate, SlicerContext, profiles::Placement,
    profiles::entry::load_settings::with_load_settings_source,
};

use super::{
    ProjectInput, ProjectSource, perimeters, profile_entry, slice_plate, source::PlateInput, state,
};

/// The output and report of one plate from the direct STL slice action.
pub struct SlicedStlPlate {
    pub id: u32,
    pub processed: ProcessedGCode,
    pub report: SlicedPlate,
}

#[derive(Debug)]
pub struct StlPlateFailure {
    pub error: Box<SliceError>,
    /// Zero if the error occurred before a plate was selected.
    pub plate: u32,
    /// The action's layer height, wall loops and sparse infill density.
    pub print_settings: Option<(f64, i32, f64)>,
    /// The failing plate's report, available once its print was applied.
    pub report: Option<Box<SlicedPlate>>,
}

impl StlPlateFailure {
    fn before_process(
        error: SliceError,
        plate: u32,
        print_settings: Option<(f64, i32, f64)>,
    ) -> Self {
        Self {
            error: Box::new(error),
            plate,
            print_settings,
            report: None,
        }
    }
}

/// Arrange and precheck the requested plates before processing any of them.
/// Pull each result in turn to process that plate and stop after the first error.
pub fn slice_stl_models_with_load_settings<'a>(
    models: Vec<ImportedStl>,
    request: LoadSettingsSliceRequest<'_>,
    selection: u32,
    mut context: SlicerContext<'a>,
    record_moves: bool,
) -> Result<impl Iterator<Item = Result<SlicedStlPlate, StlPlateFailure>> + use<'a>, StlPlateFailure>
{
    let capture_archive = request.placement == Placement::Arrange;
    if models.is_empty() {
        return Err(StlPlateFailure::before_process(
            SliceError::EmptyInput,
            0,
            None,
        ));
    }
    let model = Project::from_cli_stl_models(models);
    let mut print_settings = None;
    let mut plate = 0;
    let pending = with_load_settings_source(
        (ProjectInput::Model(&model), &[]),
        request,
        None,
        |source| {
            precheck(
                source,
                selection,
                &mut context,
                (&mut plate, &mut print_settings),
            )
        },
    )
    .map_err(|error| StlPlateFailure::before_process(error, plate, print_settings))?;
    Ok(pending
        .into_iter()
        .scan(false, move |failed, (plate, state)| {
            if *failed {
                return None;
            }
            let mut report = SlicedPlate::default();
            let result = state.report(&mut report).and_then(|()| {
                slice_plate(
                    (GenerationMetadata::default(), record_moves),
                    &mut context,
                    Some(&mut report),
                    capture_archive.then_some(super::ArchiveSource::DirectStl),
                    |_, context| {
                        perimeters::prepare_post_layer_region_perimeters(
                            PlateInput::Applied(Box::new(state)),
                            context,
                        )
                    },
                )
            });
            *failed = result.is_err();
            Some(match result {
                Ok(processed) => Ok(SlicedStlPlate {
                    id: plate,
                    processed,
                    report,
                }),
                Err(error) => Err(StlPlateFailure {
                    error: Box::new(error),
                    plate,
                    print_settings,
                    report: Some(Box::new(report)),
                }),
            })
        }))
}

fn precheck(
    mut source: ProjectSource<'_>,
    selection: u32,
    context: &mut SlicerContext<'_>,
    (current, print_settings): (&mut u32, &mut Option<(f64, i32, f64)>),
) -> Result<Vec<(u32, state::AppliedPlate)>, SliceError> {
    let profile = source.profile.expect("loaded settings supply a profile");
    let (mut project, preparation) = profile_entry::load_profile_slice(&mut source, profile, None)?;
    let plates = project
        .plates()
        .iter()
        .map(|plate| plate.id())
        .filter(|&id| selection == 0 || selection == id)
        .collect::<Vec<_>>();
    if plates.is_empty() {
        return Ok(Vec::new());
    }
    super::capabilities::validate_settings(project.settings())?;
    // OrcaSlicer.cpp:2042–2074: STL events are keyed to index 0.
    if let Some(custom_gcodes) = context.custom_gcodes.take() {
        project.replace_plate_custom_gcodes(custom_gcodes);
    }
    for &id in &plates {
        *current = id;
        project = crate::plater::fit_slice_wipe_tower(project, Some(id))?;
    }
    plates
        .into_iter()
        .map(|id| {
            *current = id;
            let state = state::apply_plate(
                project.plate_view(id, false)?,
                None,
                (true, context.max_triangle_count, context.allow_mix_temp()),
                Some((&preparation, source.overrides)),
                |project| {
                    // OrcaSlicer.cpp:6925–6932 updates shared result fields after the
                    // volume and triangle checks, before applying the per-plate config.
                    let process = &project.settings().process;
                    *print_settings = Some((
                        f64::from(process.object.layer_height.0 as f32),
                        process.region.wall_loops.0,
                        f64::from(process.region.sparse_infill_density.0 as f32),
                    ));
                },
            )?;
            Ok((id, state))
        })
        .collect()
}
