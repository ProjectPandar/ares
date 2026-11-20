//! Headless `PartPlateList::store_to_3mf_structure` plate facts.

use crate::project::raw_settings::ProjectSettingsRaw;

pub(crate) struct ArchivePlate {
    pub(crate) object_name: String,
    pub(crate) plate_id: u32,
    pub(crate) identify_id: u32,
    pub(crate) bounds: [f64; 4],
    pub(crate) area: f32,
    pub(crate) layer_height: f32,
    pub(crate) layer_count: usize,
    pub(crate) filament_color: String,
    pub(crate) filament_type: String,
    pub(crate) filament_id: String,
    pub(crate) nozzle_diameter: f32,
    pub(crate) project_settings: ProjectSettingsRaw,
}
