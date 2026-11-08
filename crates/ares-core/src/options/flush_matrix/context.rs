//! Old side of CLI preparation (`OrcaSlicer.cpp:1824–1836,3415–3441,3851`).
//! Upstream: 345efec64c7b95a05ce5cf96df0dde66a158835c.
use serde::Deserialize;

use crate::project::raw_settings::ProjectSettingsRaw;
use crate::{ConfigOverrides, NozzleVolumeType, OrcaFloat, ProjectSettings, SliceError};

#[derive(Debug)]
pub(crate) struct PreparationContext {
    pub(super) current_nozzle_count: usize,
    pub(super) colour_override: bool,
    pub(super) current_volume_types: Vec<NozzleVolumeType>,
}

#[derive(Deserialize)]
struct LoadedNozzles {
    nozzle_diameter: Option<Vec<OrcaFloat>>,
    #[serde(default)]
    nozzle_volume_type: Vec<NozzleVolumeType>,
}

impl PreparationContext {
    pub(crate) fn capture(
        raw: &ProjectSettingsRaw,
        overrides: &ConfigOverrides,
    ) -> Result<Self, SliceError> {
        let mut nozzle_raw = ProjectSettingsRaw::default();
        for key in ["nozzle_diameter", "nozzle_volume_type"] {
            if let Some(value) = raw.get(key) {
                nozzle_raw.insert(key.to_owned(), value.clone());
            }
        }
        nozzle_raw.normalize_option_scalars().map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
        nozzle_raw.handle_legacy();
        let loaded = LoadedNozzles::deserialize(nozzle_raw.into_object()).map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
        let count = loaded.nozzle_diameter.map_or(1, |values| values.len());
        Ok(Self::new(count, loaded.nozzle_volume_type, overrides))
    }

    pub(crate) fn for_overrides(mut self, overrides: &ConfigOverrides) -> Self {
        self.colour_override = overrides.contains_key("filament_colour");
        self
    }

    pub(crate) fn current(settings: &ProjectSettings, raw: &ProjectSettingsRaw) -> Self {
        Self::new(
            settings.project.print.nozzle_diameter.0.len(),
            if raw.contains_key("nozzle_volume_type") {
                settings.project.gcode.nozzle_volume_type.0.clone()
            } else {
                Vec::new()
            },
            &ConfigOverrides::default(),
        )
    }

    fn new(count: usize, mut types: Vec<NozzleVolumeType>, overrides: &ConfigOverrides) -> Self {
        if types.is_empty() {
            types.resize(count, NozzleVolumeType::Standard);
        }
        Self {
            current_nozzle_count: count,
            colour_override: overrides.contains_key("filament_colour"),
            current_volume_types: types,
        }
    }
}
