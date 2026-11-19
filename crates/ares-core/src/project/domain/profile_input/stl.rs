use crate::project::domain::object_id::ObjectIdCounter;

use super::{
    FilamentSequences, ModelSettings, PlateMetadata, Project, ProjectDocuments, ProjectInstance,
    ProjectModel, ProjectObject, SelectedProfile, SliceError, SliceInfo, SliceInfoHeader,
    Transform3d,
};

impl Project {
    pub(crate) fn with_stl_profiles(
        &self,
        profile: SelectedProfile<'_>,
        overrides: &crate::ConfigOverrides,
    ) -> Result<(Self, crate::options::flush_matrix::PreparationContext), SliceError> {
        let (settings, raw, preparation) = super::stl_settings::compose(profile, overrides)?;
        let mut project = self.copy_stl_model();
        project.settings = settings;
        project.documents.project_settings_raw = raw;
        Ok((project, preparation))
    }

    pub(crate) fn from_stl_with_profiles(
        input: &[u8],
        filename: &str,
        profile: SelectedProfile<'_>,
        overrides: &crate::ConfigOverrides,
    ) -> Result<(Self, crate::options::flush_matrix::PreparationContext), SliceError> {
        let (settings, raw, preparation) = super::stl_settings::compose(profile, overrides)?;
        let mut project = Self::from_stl(input, filename, settings, raw)?;
        // Non-project CLI import calls ensure_on_bed before arrangement (OrcaSlicer.cpp:1978).
        project.drop_instances_to_ground();
        Ok((project, preparation))
    }

    pub(crate) fn from_stl(
        input: &[u8],
        filename: &str,
        settings: crate::ProjectSettings,
        raw: crate::project::raw_settings::ProjectSettingsRaw,
    ) -> Result<Self, SliceError> {
        let volume = super::super::multipart::stl_volume(input, filename, 0)?;
        // STL.cpp creates the volume before the default instance, unlike 3MF build import.
        let mut object_ids = ObjectIdCounter::model_read();
        object_ids.add_object();
        object_ids.add_volume();
        let label_id = object_ids.add_instance();
        let instance = ProjectInstance::new([1, 0, label_id], true, true, Transform3d::IDENTITY);
        let object = ProjectObject::new(
            filename.to_owned(),
            1,
            (
                filename.to_owned(),
                String::new(),
                Default::default(),
                Default::default(),
            ),
            vec![volume],
            vec![instance],
        );
        Ok(Self::new(
            vec![ProjectModel::new(filename.to_owned(), vec![1])],
            vec![object],
            vec![PlateMetadata::new(1, vec![[1, 0, label_id]])],
            settings,
            ProjectDocuments {
                model_settings: ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: raw,
                slice_info: SliceInfo {
                    header: SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: FilamentSequences(Default::default()),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
                bbl_family_3mf: false,
                preset_file_version: None,
                preset_selection: None,
            },
        ))
    }
}
