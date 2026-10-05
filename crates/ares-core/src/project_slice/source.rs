use crate::{ConfigOverrides, project::raw_settings::ProjectSettingsRaw};

/// Input bytes and caller-selected plate/config. Loading occurs only at the
/// existing project-state seam; no intermediate archive is serialized.
#[derive(Clone, Copy)]
pub(crate) struct ProjectSource<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) plate: Option<u32>,
    pub(crate) profile: Option<ProfileInput<'a>>,
    /// User option overrides (M854); empty for the unedited document.
    pub(crate) overrides: &'a ConfigOverrides,
}

#[derive(Clone, Copy)]
pub(crate) struct ProfileInput<'a> {
    pub(crate) filename: &'a str,
    pub(crate) composed: &'a crate::profiles::ComposedProfile,
    pub(crate) raw: &'a ProjectSettingsRaw,
    /// Merged raw per selected filament (`OrcaSlicer.cpp:3301` slot overlay).
    pub(crate) filament_raws: &'a [ProjectSettingsRaw],
    pub(crate) placement: crate::profiles::Placement,
}

impl AsRef<[u8]> for ProjectSource<'_> {
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

pub(super) trait ProjectBytes<'a> {
    fn into_source(self) -> ProjectSource<'a>;
}

impl<'a> ProjectBytes<'a> for ProjectSource<'a> {
    fn into_source(self) -> ProjectSource<'a> {
        self
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> ProjectBytes<'a> for &'a T {
    fn into_source(self) -> ProjectSource<'a> {
        ProjectSource {
            bytes: self.as_ref(),
            plate: None,
            profile: None,
            overrides: &EMPTY_OVERRIDES,
        }
    }
}

static EMPTY_OVERRIDES: ConfigOverrides = ConfigOverrides::empty();

impl<'a> From<&'a [u8]> for ProjectSource<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            plate: None,
            profile: None,
            overrides: &EMPTY_OVERRIDES,
        }
    }
}
