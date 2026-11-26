//! Native CLI `--slice`: select, precheck all requested plates, then process in order.
use crate::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    ProfileSliceRequest, SliceError, SlicedPlate, SlicerContext,
    profiles::entry::{load_settings::with_load_settings_source, with_profile_source},
};

use super::{
    CliUpwardMachine, ProjectInput, ProjectSource, perimeters, profile_entry, slice_plate_to_sink,
    source::PlateInput, state,
};

pub enum CliPlatesSettings<'a> {
    Embedded {
        system_catalog: Option<&'a crate::PresetCatalogHandle>,
    },
    RetainedRefresh(&'a crate::NativeRefreshPresets),
    Catalog {
        json: &'a str,
        uptodate: bool,
    },
    Files {
        settings: &'a [LoadSettingsFile<'a>],
        filaments: &'a [LoadSettingsFile<'a>],
        catalog_json: Option<&'a str>,
        loaded_sources: Option<&'a crate::LoadedPresetSources>,
        uptodate: bool,
    },
}

pub struct CliPlatesRequest<'a> {
    pub selection: i32,
    pub filename: &'a str,
    pub settings: CliPlatesSettings<'a>,
    pub overrides: &'a ConfigOverrides,
    pub orient: bool,
    pub upward: CliUpwardMachine<'a>,
}

#[derive(Debug)]
pub enum CliPlateFailureKind {
    Selection,
    Slice(Box<SliceError>),
}

#[derive(Debug)]
pub struct CliPlateFailure {
    pub kind: CliPlateFailureKind,
    pub plate: u32,
    pub report: Box<SlicedPlate>,
}

/// Precheck every selected 3MF plate before publishing any G-code. The sink
/// receives each completed plate in Native's order; an error stops the run.
pub fn slice_cli_project_plates(
    input: &[u8],
    request: CliPlatesRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    mut publish: impl FnMut(u32, &[u8], &SlicedPlate) -> Result<(), SliceError>,
) -> Result<(), CliPlateFailure> {
    slice_project_plates_to_sink(
        (input, None),
        request,
        metadata,
        context,
        (
            |_, _| Ok(Vec::new()),
            |plate, gcode, report| publish(plate, &gcode, report),
        ),
    )
}

/// Streaming counterpart to [`slice_cli_project_plates`]: creates a sink per
/// plate via the first callback, streams finalized G-code chunks directly into
/// it, and hands the completed sink to the second callback.
pub fn slice_cli_project_plates_to_sink<S: crate::GCodeSink>(
    input: &[u8],
    request: CliPlatesRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    callbacks: (
        impl FnMut(u32, &SlicedPlate) -> Result<S, SliceError>,
        impl FnMut(u32, S, &SlicedPlate) -> Result<(), SliceError>,
    ),
) -> Result<(), CliPlateFailure> {
    slice_project_plates_to_sink((input, None), request, metadata, context, callbacks)
}

impl crate::CliPresetProject<'_> {
    pub fn slice_plates(
        self,
        request: CliPlatesRequest<'_>,
        metadata: GenerationMetadata,
        context: SlicerContext<'_>,
        mut publish: impl FnMut(u32, &[u8], &SlicedPlate) -> Result<(), SliceError>,
    ) -> Result<(), CliPlateFailure> {
        slice_project_plates_to_sink(
            self.into_parts(),
            request,
            metadata,
            context,
            (
                |_, _| Ok(Vec::new()),
                |plate, gcode, report| publish(plate, &gcode, report),
            ),
        )
    }

    pub fn slice_plates_to_sink<S: crate::GCodeSink>(
        self,
        request: CliPlatesRequest<'_>,
        metadata: GenerationMetadata,
        context: SlicerContext<'_>,
        callbacks: (
            impl FnMut(u32, &SlicedPlate) -> Result<S, SliceError>,
            impl FnMut(u32, S, &SlicedPlate) -> Result<(), SliceError>,
        ),
    ) -> Result<(), CliPlateFailure> {
        slice_project_plates_to_sink(self.into_parts(), request, metadata, context, callbacks)
    }
}

fn slice_project_plates_to_sink<S: crate::GCodeSink>(
    (input, loaded): (&[u8], Option<Box<crate::project::LoadedProfileProject>>),
    request: CliPlatesRequest<'_>,
    metadata: GenerationMetadata,
    mut context: SlicerContext<'_>,
    (mut make_sink, mut on_complete): (
        impl FnMut(u32, &SlicedPlate) -> Result<S, SliceError>,
        impl FnMut(u32, S, &SlicedPlate) -> Result<(), SliceError>,
    ),
) -> Result<(), CliPlateFailure> {
    let CliPlatesRequest {
        selection,
        filename,
        settings,
        overrides,
        orient,
        upward,
    } = request;
    let CliUpwardMachine {
        source,
        input: upward_input,
        report: upward_report,
    } = upward;
    upward_report.clear();
    let selected = u32::try_from(selection).ok().filter(|&id| id != 0);
    let mut current = 0;
    let mut invalid_selection = false;
    let mut loaded_report = SlicedPlate::default();
    let normative_check = context.normative_check();
    let skip_modified_gcodes = context.skip_modified_gcodes();
    let custom_gcodes = context.custom_gcodes.take();
    let downward = context.downward.take();
    let embedded_raw = loaded
        .as_deref()
        .and_then(crate::project::LoadedProfileProject::embedded_raw)
        .cloned();
    let run = |mut source_input: ProjectSource<'_>| {
        if let Some(profile) = source_input.profile.as_mut() {
            profile.enable_timelapse = context.enable_timelapse;
            profile.retained_refresh = profile
                .retained_refresh
                .map(|refresh| refresh.with_skip_modified_gcodes(skip_modified_gcodes));
        }
        source_input.refresh = source_input
            .refresh
            .map(|refresh| refresh.with_skip_modified_gcodes(skip_modified_gcodes));
        let mut source_input = ProjectSource {
            loaded,
            max_triangle_count: 0,
            orient,
            validate_print: true,
            normative_check,
            skip_modified_gcodes,
            custom_gcodes,
            downward: None,
            ..source_input
        };
        let selected_printer = source_input
            .profile
            .map(super::source::ProfileInput::printer_name);
        let (project, preparation) = if let Some(profile) = source_input.profile {
            profile_entry::load_profile_slice(&mut source_input, profile, Some(upward_report))?
        } else {
            state::load_project_source(ProjectSource {
                max_triangle_count: 0,
                upward: Some(CliUpwardMachine {
                    source,
                    input: upward_input,
                    report: upward_report,
                }),
                ..source_input
            })?
        };
        crate::downward::check_project(&project, selected, downward);
        let mut project = crate::plater::initialize_slice_plates(project);
        let plates = project
            .plates()
            .iter()
            .map(|plate| plate.id())
            .filter(|id| selected.is_none_or(|selected| selected == *id))
            .collect::<Vec<_>>();
        if selection < 0 || plates.is_empty() {
            invalid_selection = true;
            return Err(SliceError::InvalidInput(
                "invalid CLI plate selection".to_owned(),
            ));
        }
        for &plate in &plates {
            current = plate;
            project = crate::plater::fit_slice_wipe_tower(project, Some(plate))?;
        }
        let mut applied = Vec::with_capacity(plates.len());
        for plate in plates {
            current = plate;
            let view = project.plate_view(plate, selected.is_some())?;
            let state = state::apply_plate(
                view,
                selected_printer,
                (true, 0, false),
                Some((&preparation, overrides)),
                |_| {},
            )?;
            let mut report = SlicedPlate::default();
            state.report(&mut report)?;
            loaded_report.clone_from(&report);
            applied.push((plate, state, report));
        }
        for (plate, state, mut plate_report) in applied {
            current = plate;
            let mut sink = make_sink(plate, &plate_report)?;
            let result = slice_plate_to_sink(
                (metadata, false),
                (&mut context, &mut sink),
                (Some(&mut plate_report), None),
                None,
                |_, context| {
                    perimeters::prepare_post_layer_region_perimeters(
                        PlateInput::Applied(Box::new(state)),
                        context,
                    )
                },
            );
            loaded_report = plate_report;
            result?;
            on_complete(plate, sink, &loaded_report)?;
        }
        Ok(())
    };
    let result = overrides
        .validate_cli_config()
        .and_then(|()| match settings {
            CliPlatesSettings::Embedded { system_catalog } => run(ProjectSource {
                input: ProjectInput::SavedCliBytes(input),
                max_triangle_count: 0,
                plate: selected,
                overrides,
                system_catalog,
                ..ProjectSource::from(input)
            }),
            CliPlatesSettings::RetainedRefresh(refresh) => run(ProjectSource {
                input: ProjectInput::SavedCliBytes(input),
                plate: selected,
                overrides,
                refresh: Some(crate::profiles::ProjectPresetRefresh::native(
                    refresh,
                    skip_modified_gcodes,
                )),
                ..ProjectSource::from(input)
            }),
            CliPlatesSettings::Catalog { json, uptodate } => with_profile_source(
                input,
                ProfileSliceRequest {
                    filename,
                    catalog_json: json,
                    selection_json: if uptodate {
                        r#"{"from_project":true,"uptodate":true}"#
                    } else {
                        r#"{"from_project":true,"uptodate":false}"#
                    },
                    plate: selected,
                    overrides,
                    placement: Placement::None,
                },
                crate::Composition::Cli,
                run,
            ),
            CliPlatesSettings::Files {
                settings,
                filaments,
                catalog_json,
                loaded_sources,
                uptodate,
            } => {
                let mut report = SlicedPlate::default();
                with_load_settings_source(
                    (ProjectInput::Bytes(input), input, embedded_raw.as_ref()),
                    LoadSettingsSliceRequest {
                        filename,
                        catalog_json,
                        loaded_sources,
                        uptodate,
                        settings,
                        filaments,
                        plate: selected,
                        overrides,
                        placement: Placement::None,
                        orient,
                        sliced_plate: &mut report,
                    },
                    None,
                    run,
                )
            }
        });
    result.map_err(|error| CliPlateFailure {
        kind: if invalid_selection {
            CliPlateFailureKind::Selection
        } else {
            CliPlateFailureKind::Slice(Box::new(error))
        },
        plate: if invalid_selection { 0 } else { current },
        report: Box::new(loaded_report),
    })
}
