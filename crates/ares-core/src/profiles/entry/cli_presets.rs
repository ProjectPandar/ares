use super::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, load_settings, slice_cli_profile_source,
};
use crate::{
    ConfigOverrides, GenerationMetadata, InputFormat, LoadSettingsRequest, NativeRefreshPresets,
    ProcessedGCode, Project, SliceError, SlicerContext,
    project::{LoadedProfileProject, SelectedProfile, export_prepared_settings},
};

/// An input admitted before opening any externally supplied presets.
pub struct CliPresetProject<'a> {
    input: &'a [u8],
    loaded: Option<Box<LoadedProfileProject>>,
}

impl<'a> CliPresetProject<'a> {
    /// Import 3MF geometry and check embedded scripts before preset reads
    /// (`OrcaSlicer.cpp:1755–1810,1953–2024`). Other formats are unchanged.
    pub fn admit(input: &'a [u8], context: &SlicerContext<'_>) -> Result<Self, SliceError> {
        let loaded = if InputFormat::detect(input) == InputFormat::ThreeMf {
            Some(Box::new(LoadedProfileProject::load(
                input,
                context.normative_check(),
            )?))
        } else {
            None
        };
        Ok(Self { input, loaded })
    }

    pub fn input(&self) -> &'a [u8] {
        self.input
    }

    /// Whether the imported 3MF producer belongs to the Bambu/Orca family.
    pub fn is_bbl_family_3mf(&self) -> bool {
        self.loaded
            .as_deref()
            .is_some_and(LoadedProfileProject::is_bbl_family_3mf)
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
        let (composed, names) =
            crate::profiles::load_settings::compose_project_load_settings(self.input, request)?;
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
