//! Headless `PartPlateList::store_to_3mf_structure` plate facts.

use crate::project::raw_settings::ProjectSettingsRaw;

pub(crate) struct ArchivePlate {
    pub(crate) objects: Vec<crate::SlicedArchiveObject>,
    pub(crate) layer_count: usize,
    pub(crate) filament_color: String,
    pub(crate) filament_type: String,
    pub(crate) filament_id: String,
    pub(crate) nozzle_diameter: f32,
    pub(crate) project_settings: ProjectSettingsRaw,
}
