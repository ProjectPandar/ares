use super::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, load_settings, slice_cli_profile_source,
};
use crate::{
    GenerationMetadata, InputFormat, NativeRefreshPresets, ProcessedGCode, SliceError,
    SlicerContext, project::LoadedProfileProject,
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
