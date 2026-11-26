use super::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, load_settings, slice_cli_profile_source,
};
use crate::{
    ConfigOverrides, ConfigSubstitution, GenerationMetadata, InputFormat, LoadSettingsRequest,
    NativeRefreshPresets, PresetCatalogHandle, ProcessedGCode, Project, SliceError, SlicerContext,
    UpwardMachineSource,
    project::{
        LoadedProfileProject, SelectedProfile, export_loaded_project_settings,
        export_prepared_settings,
    },
};

/// An input admitted before opening any externally supplied presets.
pub struct CliPresetProject<'a> {
    input: &'a [u8],
    loaded: Option<Box<LoadedProfileProject>>,
}

impl<'a> CliPresetProject<'a> {
    /// Open and parse a 3MF project once, optionally checking its application
    /// version (`bbs_3mf.cpp:1537-1550, 1944-1947`) before reading meshes,
    /// without running the normative `post_process` check yet.
    pub fn open(input: &'a [u8], allow_newer_file: bool) -> Result<Self, SliceError> {
        let loaded = if InputFormat::detect(input) == InputFormat::ThreeMf {
            Some(Box::new(LoadedProfileProject::open(
                input,
                !allow_newer_file,
            )?))
        } else {
            None
        };
        Ok(Self { input, loaded })
    }

    /// Import 3MF geometry and check embedded scripts before preset reads
    /// (`OrcaSlicer.cpp:1755–1810,1953–2024`). Other formats are unchanged.
    pub fn admit(input: &'a [u8], context: &SlicerContext<'_>) -> Result<Self, SliceError> {
        let project = Self::open(input, true)?;
        project.check_post_process(context)?;
        Ok(project)
    }

    /// Run the embedded `post_process` admission gate when enabled by `context`.
    pub fn check_post_process(&self, context: &SlicerContext<'_>) -> Result<(), SliceError> {
        if context.normative_check()
            && let Some(loaded) = self.loaded.as_deref()
        {
            loaded.validate_post_process()?;
        }
        Ok(())
    }

    pub fn input(&self) -> &'a [u8] {
        self.input
    }

    /// Embedded project-settings substitutions captured during the single 3MF load.
    pub fn substitutions(&self) -> &[ConfigSubstitution] {
        self.loaded
            .as_deref()
            .map_or(&[], LoadedProfileProject::substitutions)
    }

    /// Original BBL machine identity captured during the single 3MF load.
    pub fn upward_source(&self) -> Result<Option<&UpwardMachineSource>, SliceError> {
        match self.loaded.as_deref() {
            Some(loaded) => loaded.upward_source(),
            None => Ok(None),
        }
    }

    /// Moves out the original BBL machine identity captured during the single 3MF load.
    pub fn take_upward_source(&mut self) -> Result<Option<UpwardMachineSource>, SliceError> {
        match self.loaded.as_deref_mut() {
            Some(loaded) => loaded.take_upward_source(),
            None => Ok(None),
        }
    }

    /// Whether the imported 3MF producer belongs to the Bambu/Orca family.
    pub fn is_bbl_family_3mf(&self) -> bool {
        self.loaded
            .as_deref()
            .is_some_and(LoadedProfileProject::is_bbl_family_3mf)
    }

    /// Export saved project settings using the already-loaded 3MF and an
    /// installation preset catalog, without reopening the archive.
    pub fn export_settings_with_catalog(
        self,
        catalog: &PresetCatalogHandle,
    ) -> Result<Vec<u8>, SliceError> {
        let Some(loaded) = self.loaded else {
            return crate::project::export_project_settings_with_catalog(self.input, catalog);
        };
        let (project, preparation) = loaded.into_direct_project(&ConfigOverrides::default())?;
        export_loaded_project_settings(project, preparation, Some(catalog))
    }

    /// Save the composed configuration of a BBL 3MF without slicing a plate.
    pub fn export_settings_with_load_settings(
        self,
        request: &LoadSettingsRequest<'_>,
        enable_timelapse: bool,
    ) -> Result<Vec<u8>, SliceError> {
        if !self.is_bbl_family_3mf() {
            return Err(SliceError::InvalidInput(
                "expected a BBL-family 3MF".to_owned(),
            ));
        }
        let embedded_raw = self
            .loaded
            .as_deref()
            .and_then(LoadedProfileProject::embedded_raw);
        let (composed, names) = crate::profiles::load_settings::compose_project_load_settings(
            (self.input, embedded_raw),
            request,
        )?;
        let profile = SelectedProfile {
            settings: composed.profile.settings(),
            profile_raw: &composed.raw,
            filament_raws: &composed.filament_raws,
            validate_post_process: true,
            enable_timelapse,
            load_names: Some(&names),
            retained_refresh: None,
            full_fff_config: None,
        };
        let overrides = ConfigOverrides::default();
        let (project, preparation) = Project::from_3mf_with_profiles(
            self.into_parts(),
            profile,
            &overrides,
            (None, &crate::plater::NO_EDITS),
            None,
        )?;
        // Config.cpp's preset deserializer binds vector-valued preset tokens
        // as vectors before ConfigBase::save_to_json calls vserialize().
        let mut raw = project.documents().project_settings_raw.clone();
        raw.normalize_wire_vector_shapes()
            .map_err(SliceError::InvalidInput)?;
        export_prepared_settings(
            &project,
            raw,
            &preparation,
            &overrides,
            project.cli_filament_count(),
        )
    }

    pub(crate) fn into_parts(self) -> (&'a [u8], Option<Box<LoadedProfileProject>>) {
        (self.input, self.loaded)
    }

    pub fn slice_with_profiles(
        self,
        request: CliProfileSliceRequest<'_>,
        metadata: GenerationMetadata,
        context: SlicerContext<'_>,
    ) -> Result<ProcessedGCode, SliceError> {
        slice_cli_profile_source((self.input, self.loaded), request, metadata, context)
    }

    pub fn slice_with_load_settings(
        self,
        request: LoadSettingsSliceRequest<'_>,
        refresh: Option<&NativeRefreshPresets>,
        metadata: GenerationMetadata,
        context: SlicerContext<'_>,
    ) -> Result<Vec<u8>, SliceError> {
        load_settings::slice_load_settings_source(
            self.into_parts(),
            request,
            refresh,
            (metadata, false),
            context,
        )
        .map(|processed| processed.gcode)
    }

    pub fn slice_with_load_settings_processed(
        self,
        request: LoadSettingsSliceRequest<'_>,
        metadata: GenerationMetadata,
        context: SlicerContext<'_>,
        record_moves: bool,
    ) -> Result<ProcessedGCode, SliceError> {
        load_settings::slice_load_settings_source(
            self.into_parts(),
            request,
            None,
            (metadata, record_moves),
            context,
        )
    }
}
