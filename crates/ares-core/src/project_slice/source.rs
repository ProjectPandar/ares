use crate::{ConfigOverrides, project::raw_settings::ProjectSettingsRaw};

/// Input bytes and caller-selected plate/config. Loading occurs only at the
/// existing project-state seam; no intermediate archive is serialized.
pub(crate) struct ProjectSource<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) plate: Option<u32>,
    pub(crate) profile: Option<ProfileInput<'a>>,
    pub(crate) refresh: Option<crate::profiles::ProjectPresetRefresh<'a>>,
    /// User option overrides (M854); empty for the unedited document.
    pub(crate) overrides: &'a ConfigOverrides,
    /// Adapter boundary: apply once, validate, then process (`Print.hpp:911,935`).
    pub(crate) validate_print: bool,
    /// `DynamicPrintConfig::validate` of the loaded config, which the CLI rejects on
    /// (`OrcaSlicer.cpp:4277-4282`); the GUI only warns at load (`Plater.cpp:9466-9481`).
    pub(crate) validate_config: bool,
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
    pub triangle_count: usize,
    /// The f32-stored value widened to f64.
    pub layer_height: f64,
    pub wall_loops: i32,
    pub sparse_infill_density: f64,
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
    pub(crate) placement: crate::profiles::Placement,
    /// CLI composite entries set this; the profile path checks the
    /// embedded `post_process` on it before overlay (O5).
    pub(crate) validate_print: bool,
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
            refresh: None,
            overrides: &EMPTY_OVERRIDES,
            validate_print: false,
            validate_config: true,
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
            bytes,
            plate: None,
            profile: None,
            refresh: None,
            overrides: &EMPTY_OVERRIDES,
            validate_print: false,
            validate_config: true,
            upward: None,
            sliced_plate: None,
            edits: &crate::plater::NO_EDITS,
        }
    }
}
