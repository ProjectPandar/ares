use crate::project::{
    filament_sequence::FilamentSequences, model_settings::ModelSettings, plate::PlateJson,
    slice_info::SliceInfo,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PlateMetadata {
    id: u32,
    instances: Vec<[u32; 3]>,
}

impl PlateMetadata {
    pub(crate) fn new(id: u32, instances: Vec<[u32; 3]>) -> Self {
        Self { id, instances }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn instances(&self) -> &[[u32; 3]] {
        &self.instances
    }
}

#[derive(Debug)]
pub(crate) struct ProjectDocuments {
    pub model_settings: ModelSettings,
    /// Raw `Metadata/project_settings.config` key/values, used to drive the
    /// config block key set (`GCode::append_full_config`).
    pub project_settings_raw: std::collections::BTreeMap<String, serde_json::Value>,
    pub slice_info: SliceInfo,
    pub filament_sequences: FilamentSequences,
    pub plate_documents: Vec<PlateJson>,
    pub has_painted_layer_height_profile: bool,
}

impl ProjectDocuments {
    pub(crate) fn clone_shallow(&self) -> Self {
        Self {
            model_settings: self.model_settings.clone(),
            project_settings_raw: self.project_settings_raw.clone(),
            slice_info: self.slice_info.clone(),
            filament_sequences: self.filament_sequences.clone(),
            plate_documents: self.plate_documents.clone(),
            has_painted_layer_height_profile: self.has_painted_layer_height_profile,
        }
    }
}
