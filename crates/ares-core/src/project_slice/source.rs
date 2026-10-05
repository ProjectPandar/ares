use crate::{ProjectSettings, project::raw_settings::ProjectSettingsRaw};

/// Input bytes and caller-selected plate/config. Loading occurs only at the
/// existing project-state seam; no intermediate archive is serialized.
#[derive(Clone, Copy)]
pub(crate) struct ProjectSource<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) plate: Option<u32>,
    pub(crate) profile: Option<ProfileInput<'a>>,
}

#[derive(Clone, Copy)]
pub(crate) struct ProfileInput<'a> {
    pub(crate) filename: &'a str,
    pub(crate) settings: &'a ProjectSettings,
    pub(crate) raw: &'a ProjectSettingsRaw,
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
        }
    }
}

impl<'a> From<&'a [u8]> for ProjectSource<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            plate: None,
            profile: None,
        }
    }
}
