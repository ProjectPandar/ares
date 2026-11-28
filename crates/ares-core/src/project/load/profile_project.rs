use super::{
    ConfigSubstitution, MODEL_RELATIONSHIP_TYPE, PackagePath, Project, ProjectArchive,
    ProjectSettingsRaw, ROOT_RELATIONSHIPS_PATH, Relationships, SliceError, XmlRole,
    deserialize_xml, load_project_from_archive, load_project_raw, settings,
};
use crate::project::UpwardMachineSource;

pub(super) struct LoadedProjectData {
    pub project: Project,
    pub preparation: crate::options::flush_matrix::PreparationContext,
    pub sparse_settings: ProjectSettingsRaw,
    pub embedded_raw: Option<ProjectSettingsRaw>,
    pub substitutions: Vec<ConfigSubstitution>,
    pub upward_source: Result<Option<UpwardMachineSource>, String>,
}

type PrefetchedRoot = (Option<Vec<u8>>, Option<(PackagePath, Vec<u8>)>);

/// Reads `_rels/.rels` and the root model once before full package parsing to
/// reject unsupported newer application versions early while returning both
/// buffers for reuse.
pub(super) fn prefetch_and_check_version(
    archive: &mut ProjectArchive<'_>,
) -> Result<PrefetchedRoot, SliceError> {
    let root = PackagePath::root();
    let Ok(rels_bytes) = root
        .resolve(ROOT_RELATIONSHIPS_PATH)
        .and_then(|path| archive.read(&path))
    else {
        return Ok((None, None));
    };
    let Ok(rels) = deserialize_xml::<Relationships>(&rels_bytes, XmlRole::Relationships) else {
        return Ok((Some(rels_bytes), None));
    };
    let Ok(model_path) = rels.resolve_required(&root, MODEL_RELATIONSHIP_TYPE) else {
        return Ok((Some(rels_bytes), None));
    };
    let Ok(model_bytes) = archive.read(&model_path) else {
        return Ok((Some(rels_bytes), None));
    };
    if let Err(error @ SliceError::FileVersionNotSupported) =
        super::super::preset_version::check_root_model_bytes(&model_bytes)
    {
        return Err(error);
    }
    Ok((Some(rels_bytes), Some((model_path, model_bytes))))
}

pub(crate) struct LoadedProfileProject {
    project: Project,
    sparse_settings: ProjectSettingsRaw,
    embedded_raw: Option<ProjectSettingsRaw>,
    substitutions: Vec<ConfigSubstitution>,
    upward_source: Result<Option<UpwardMachineSource>, String>,
    preparation: crate::options::flush_matrix::PreparationContext,
}

impl LoadedProfileProject {
    pub(crate) fn is_bbl_family_3mf(&self) -> bool {
        self.project.documents().bbl_family_3mf
    }

    pub(crate) fn project(&self) -> &Project {
        &self.project
    }

    pub(crate) fn substitutions(&self) -> &[ConfigSubstitution] {
        &self.substitutions
    }

    pub(crate) fn embedded_raw(&self) -> Option<&ProjectSettingsRaw> {
        self.embedded_raw.as_ref()
    }

    pub(crate) fn take_embedded_raw(&mut self) -> Option<ProjectSettingsRaw> {
        self.embedded_raw.take()
    }

    pub(crate) fn upward_source(&self) -> Result<Option<&UpwardMachineSource>, SliceError> {
        match &self.upward_source {
            Ok(source) => Ok(source.as_ref()),
            Err(message) => Err(SliceError::InvalidInput(message.clone())),
        }
    }

    pub(crate) fn take_upward_source(&mut self) -> Result<Option<UpwardMachineSource>, SliceError> {
        match &mut self.upward_source {
            Ok(source) => Ok(source.take()),
            Err(message) => Err(SliceError::InvalidInput(message.clone())),
        }
    }

    pub(crate) fn validate_post_process(&self) -> Result<(), SliceError> {
        crate::project_slice::validate_post_process(&self.project)
    }

    pub(crate) fn open(input: &[u8], check_version: bool) -> Result<Self, SliceError> {
        let data = load_project_raw(
            input,
            &crate::ConfigOverrides::empty(),
            true,
            (true, check_version),
        )?;
        Ok(Self::from_data(data))
    }

    pub(crate) fn open_from_archive(
        archive: &mut ProjectArchive<'_>,
        check_version: bool,
    ) -> Result<Self, SliceError> {
        let data = load_project_from_archive(
            archive,
            &crate::ConfigOverrides::empty(),
            true,
            (true, check_version),
        )?;
        Ok(Self::from_data(data))
    }

    pub(crate) fn open_for_export(archive: &mut ProjectArchive<'_>) -> Result<Self, SliceError> {
        let data = load_project_from_archive(
            archive,
            &crate::ConfigOverrides::default(),
            false,
            (true, false),
        )?;
        Ok(Self::from_data(data))
    }

    fn from_data(data: LoadedProjectData) -> Self {
        Self {
            project: data.project,
            sparse_settings: data.sparse_settings,
            embedded_raw: data.embedded_raw,
            substitutions: data.substitutions,
            upward_source: data.upward_source,
            preparation: data.preparation,
        }
    }

    pub(crate) fn into_direct_project(
        self,
        overrides: &crate::ConfigOverrides,
    ) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
        let Self {
            mut project,
            sparse_settings,
            preparation,
            ..
        } = self;
        if !overrides.is_empty() {
            let (raw, settings) = settings::resolve(sparse_settings, overrides, true)?;
            project.set_embedded_settings(raw, settings);
        }
        Ok((project, preparation))
    }

    pub(crate) fn into_profile_project(
        self,
    ) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
        let Self {
            mut project,
            sparse_settings,
            preparation,
            ..
        } = self;
        let (raw, settings) =
            settings::resolve(sparse_settings, &crate::ConfigOverrides::empty(), false)?;
        project.set_embedded_settings(raw, settings);
        Ok((project, preparation))
    }
}
