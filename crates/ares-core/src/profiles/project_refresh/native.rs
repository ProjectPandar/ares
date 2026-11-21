use crate::{
    PresetCatalogHandle, ProfileKind, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// A flattened Native refresh file. Unlike a resolved catalog record, an
/// absent option in this file is not a supplied collection default.
pub struct NativeRefreshFile {
    kind: ProfileKind,
    name: String,
    pub(super) raw: ProjectSettingsRaw,
}

impl NativeRefreshFile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, SliceError> {
        let bytes = crate::profiles::without_utf8_bom(bytes);
        let mut raw: ProjectSettingsRaw = serde_json::from_slice(bytes)
            .map_err(|error| SliceError::InvalidInput(format!("invalid preset file: {error}")))?;
        let text = |key| match raw.get(key) {
            Some(RawConfigValue::Text(text)) => text.as_str(),
            _ => "",
        };
        let kind = ProfileKind::parse(text("type"))?;
        let name = text("name").to_owned();
        if name.is_empty() || !matches!(text("from"), "system" | "User" | "user") {
            return Err(SliceError::InvalidInput(
                "invalid preset name or source".into(),
            ));
        }
        raw.handle_legacy();
        let mut options = ProjectSettingsRaw::default();
        options.extend(
            raw.entries()
                .filter(|(key, _)| super::preset_owns_option(kind, key))
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        options
            .normalize_wire_vector_shapes()
            .map_err(SliceError::InvalidInput)?;
        Ok(Self {
            kind,
            name,
            raw: options,
        })
    }

    pub fn kind(&self) -> ProfileKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn upward_machines(&self) -> Vec<String> {
        super::preset::strings(&self.raw, "upward_compatible_machine")
    }
}

/// Native resource/file lookup results, in project filament slot order.
#[derive(Default)]
pub struct NativeRefreshPresets {
    pub machine: Option<NativeRefreshFile>,
    pub process: Option<NativeRefreshFile>,
    pub filaments: Vec<Option<NativeRefreshFile>>,
    pub system_catalog: Option<PresetCatalogHandle>,
}

impl NativeRefreshPresets {
    pub(super) fn file(&self, kind: ProfileKind, slot: usize) -> Option<&NativeRefreshFile> {
        match kind {
            ProfileKind::Machine => self.machine.as_ref(),
            ProfileKind::Process => self.process.as_ref(),
            ProfileKind::Filament => self.filaments.get(slot).and_then(Option::as_ref),
        }
    }
}
