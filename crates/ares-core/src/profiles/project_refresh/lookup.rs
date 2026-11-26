use super::super::{ProfileKind, catalog::Catalog};
use super::{NativeRefreshPresets, preset};
use crate::{SliceError, project::raw_settings::ProjectSettingsRaw};

#[derive(Clone, Copy)]
pub(super) enum PresetLookup<'a> {
    Catalog {
        catalog: &'a Catalog,
        skip_modified_gcodes: bool,
    },
    Native {
        presets: &'a NativeRefreshPresets,
        skip_modified_gcodes: bool,
    },
}

impl PresetLookup<'_> {
    pub(super) fn skip_modified_gcodes(self) -> bool {
        match self {
            Self::Catalog {
                skip_modified_gcodes,
                ..
            }
            | Self::Native {
                skip_modified_gcodes,
                ..
            } => skip_modified_gcodes,
        }
    }

    pub(super) fn is_native_file(self, kind: ProfileKind, slot: usize) -> bool {
        matches!(self, Self::Native { presets, .. } if presets.file(kind, slot).is_some())
    }

    pub(super) fn native_file(self, kind: ProfileKind, slot: usize) -> Option<ProjectSettingsRaw> {
        match self {
            Self::Native { presets, .. } => presets.file(kind, slot).map(|file| file.raw.clone()),
            Self::Catalog { .. } => None,
        }
    }

    pub(super) fn system(
        self,
        kind: ProfileKind,
        name: &str,
        slot: usize,
    ) -> Result<Option<ProjectSettingsRaw>, SliceError> {
        let catalog = match self {
            Self::Catalog { catalog, .. } => Some(catalog),
            Self::Native { presets, .. } => {
                if let Some(file) = presets.file(kind, slot) {
                    return Ok(Some(file.raw.clone()));
                }
                presets
                    .system_catalog
                    .as_ref()
                    .map(|handle| handle.catalog())
            }
        };
        match catalog {
            Some(catalog) => preset::system_config(catalog, kind, name),
            None => Ok(None),
        }
    }
}
