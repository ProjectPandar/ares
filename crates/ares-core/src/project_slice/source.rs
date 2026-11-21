use crate::{ConfigOverrides, project::raw_settings::ProjectSettingsRaw};

/// Input bytes and caller-selected plate/config. Loading occurs only at the
/// existing project-state seam; no intermediate archive is serialized.
pub(crate) struct ProjectSource<'a> {
    pub(crate) input: ProjectInput<'a>,
    pub(crate) plate: Option<u32>,
    pub(crate) profile: Option<ProfileInput<'a>>,
    pub(crate) refresh: Option<crate::profiles::ProjectPresetRefresh<'a>>,
    pub(crate) system_catalog: Option<&'a crate::PresetCatalogHandle>,
    /// User option overrides (M854); empty for the unedited document.
    pub(crate) overrides: &'a ConfigOverrides,
    /// Adapter boundary: apply once, validate, then process (`Print.hpp:911,935`).
    pub(crate) validate_print: bool,
    /// `DynamicPrintConfig::validate` of the loaded config, which the CLI rejects on
    /// (`OrcaSlicer.cpp:4277-4282`); the GUI only warns at load (`Plater.cpp:9466-9481`).
    pub(crate) validate_config: bool,
    pub(crate) orient: bool,
    pub(crate) upward: Option<CliUpwardMachine<'a>>,
    /// The CLI's success report values, filled once the plate is set up.
    pub(crate) sliced_plate: Option<&'a mut SlicedPlate>,
    /// Plater edits applied to the loaded model (empty for a plain slice).
    pub(crate) edits: &'a crate::plater::PlaterEdits,
}

/// CLI byte resources and the report snapshot written at CLI:2656. The
/// source is obtained from core before the adapter reads its resource file.
pub struct CliUpwardMachine<'a> {
    pub source: Option<crate::UpwardMachineSource>,
    pub input: crate::UpwardMachineInput<'a>,
    pub report: &'a mut Vec<String>,
}

/// The plate values of the CLI's success report (`OrcaSlicer.cpp:8126-8136` at v2.5.0-beta
/// `345efec64`): `sliced_plate_info.triangle_count` (`:6576-6640`) and `sliced_info`'s
/// `layer_height`, `wall_loops`, `sparse_infill_density` (`:6926-6931`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlicedPlate {
    /// `PrintObject` warnings captured before export, including on later failure.
    pub warnings: Vec<crate::PrintWarning>,
    /// Original upward vector committed after compatibility (OrcaSlicer.cpp:3005).
    pub upward_machines: Vec<String>,
    pub triangle_count: usize,
    /// The f32-stored value widened to f64.
    pub layer_height: f64,
    pub wall_loops: i32,
    pub sparse_infill_density: f64,
    /// Object bounding-box dimensions for the selected plate in millimetres.
    pub object_size: [f64; 3],
}

#[derive(Clone, Copy)]
pub(crate) struct ProfileInput<'a> {
    pub(crate) filename: &'a str,
    pub(crate) composed: &'a crate::profiles::ComposedProfile,
    pub(crate) raw: &'a ProjectSettingsRaw,
    /// Merged raw per selected filament (`OrcaSlicer.cpp:3301` slot overlay).
    pub(crate) filament_raws: &'a [ProjectSettingsRaw],
    /// Loaded `--load-settings` machine/process descriptors for the
    /// `OrcaSlicer.cpp:2575-2620` loaded-names adjudication; `None` on
    /// the catalog/profile paths.
    pub(crate) load_names: Option<&'a super::compatibility::LoadedPresetNames>,
    pub(crate) retained_refresh: Option<crate::profiles::ProjectPresetRefresh<'a>>,
    pub(crate) placement: crate::profiles::Placement,
    /// CLI composite entries set this; the profile path checks the
    /// embedded `post_process` on it before overlay (O5).
    pub(crate) validate_print: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum ProjectInput<'a> {
    Bytes(&'a [u8]),
    Model(&'a crate::Project),
}

impl ProjectInput<'_> {
    pub(crate) fn load(
        self,
        overrides: &ConfigOverrides,
        refresh: Option<crate::profiles::ProjectPresetRefresh<'_>>,
        system_catalog: Option<&crate::PresetCatalogHandle>,
        validate_print: bool,
    ) -> Result<
        (
            crate::Project,
            crate::options::flush_matrix::PreparationContext,
        ),
        crate::SliceError,
    > {
        match self {
            Self::Bytes(bytes) => {
                let (mut project, preparation) = match refresh {
                    Some(refresh) => refresh.load(bytes, overrides, validate_print)?,
                    None if system_catalog.is_some() => {
                        crate::project::load_project_for_profiles(bytes)?
                    }
                    None => crate::project::load_project_with_preparation(bytes, overrides)?,
                };
                if refresh.is_none()
                    && let Some(system_catalog) = system_catalog
                {
                    apply_direct_catalog(&mut project, system_catalog, overrides)?;
                }
                Ok((project, preparation.for_overrides(overrides)))
            }
            Self::Model(model) => {
                if refresh.is_some() {
                    return Err(crate::SliceError::InvalidInput(
                        "project preset refresh requires a 3MF input".to_owned(),
                    ));
                }
                let preparation = crate::options::flush_matrix::PreparationContext::capture(
                    &crate::project::raw_settings::ProjectSettingsRaw::default(),
                    overrides,
                )?;
                let mut project = model.copy_stl_model();
                if !overrides.is_empty() {
                    project.apply_project_preset_refresh(
                        model.documents().project_settings_raw.clone(),
                        overrides,
                    )?;
                }
                Ok((project, preparation))
            }
        }
    }
}

fn apply_direct_catalog(
    project: &mut crate::Project,
    catalog: &crate::PresetCatalogHandle,
    overrides: &ConfigOverrides,
) -> Result<(), crate::SliceError> {
    let mut raw = if project.documents().bbl_family_3mf {
        catalog.refresh_direct(&project.documents().project_settings_raw)?
    } else {
        project.documents().project_settings_raw.clone()
    };
    crate::profiles::apply_retained_preset_defaults(&mut raw, None)?;
    project.apply_project_preset_refresh(raw, overrides)
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
            input: ProjectInput::Bytes(self.as_ref()),
            plate: None,
            profile: None,
            refresh: None,
            system_catalog: None,
            overrides: &EMPTY_OVERRIDES,
            validate_print: false,
            validate_config: true,
            orient: false,
            upward: None,
            sliced_plate: None,
            edits: &crate::plater::NO_EDITS,
        }
    }
}

static EMPTY_OVERRIDES: ConfigOverrides = ConfigOverrides::empty();

impl<'a> From<&'a [u8]> for ProjectSource<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            input: ProjectInput::Bytes(bytes),
            plate: None,
            profile: None,
            refresh: None,
            system_catalog: None,
            overrides: &EMPTY_OVERRIDES,
            validate_print: false,
            validate_config: true,
            orient: false,
            upward: None,
            sliced_plate: None,
            edits: &crate::plater::NO_EDITS,
        }
    }
}
