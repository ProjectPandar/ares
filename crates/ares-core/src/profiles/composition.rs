mod filament;
mod metadata;

use super::{
    fragment::{ProfileFragment, ProfileKind, merge_profile_fragments},
    inheritance::{MergedProfile, MergedProfileMetadata, UnselectedKind, unselected_profile},
};
use crate::{
    OrcaInt, OrcaInts, OrcaString, OrcaStrings, PresetMetadata, ProjectRuntimeOptions,
    ProjectSettings, SliceError,
};

use self::filament::compose_filaments;
pub use self::metadata::ProfileGroupMetadata;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileSelection {
    process: Option<String>,
    machine: Option<String>,
    filaments: Vec<String>,
}

impl ProfileSelection {
    pub fn new(
        process: impl Into<String>,
        machine: impl Into<String>,
        filaments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SliceError> {
        Self::new_partial(Some(process), Some(machine), filaments)
    }

    /// A load-settings selection: `None` machine/process marks the kind
    /// as not loaded — it contributes no overlay and the embedded config
    /// of that kind is retained (`OrcaSlicer.cpp:2009-2106` arms run per
    /// loaded file only). Without embedded project state a filament
    /// selection is required.
    pub fn new_partial(
        process: Option<impl Into<String>>,
        machine: Option<impl Into<String>>,
        filaments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SliceError> {
        let process = process.map(Into::into);
        let machine = machine.map(Into::into);
        let filaments = filaments.into_iter().map(Into::into).collect::<Vec<_>>();
        if process.as_ref().is_some_and(String::is_empty)
            || machine.as_ref().is_some_and(String::is_empty)
            || filaments.is_empty()
            || filaments.iter().any(String::is_empty)
        {
            return Err(SliceError::InvalidInput(
                "profile selection must include process, machine, and filaments".to_owned(),
            ));
        }

        Ok(Self {
            process,
            machine,
            filaments,
        })
    }

    pub(super) fn machine_for_project(
        machine: String,
        embedded: &crate::project::raw_settings::ProjectSettingsRaw,
    ) -> Result<Self, SliceError> {
        use crate::project::raw_settings::RawConfigValue;
        let nonempty_name = |value: &RawConfigValue| matches!(value, RawConfigValue::Text(name) if !name.is_empty());
        let process = embedded
            .get("print_settings_id")
            .and_then(|value| value.element(0))
            .is_some_and(nonempty_name);
        let filaments = embedded.get("filament_settings_id").is_some_and(|value| {
            value.len() > 0
                && (0..value.len()).all(|index| nonempty_name(value.element(index).unwrap()))
        });
        if machine.is_empty() || !process || !filaments {
            return Err(SliceError::InvalidInput(
                "machine-only settings require an embedded process and filaments".to_owned(),
            ));
        }
        Ok(Self {
            process: None,
            machine: Some(machine),
            filaments: Vec::new(),
        })
    }

    pub fn process(&self) -> Option<&str> {
        self.process.as_deref()
    }

    pub fn machine(&self) -> Option<&str> {
        self.machine.as_deref()
    }

    pub fn filaments(&self) -> &[String] {
        &self.filaments
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComposedProfile {
    settings: ProjectSettings,
    metadata: ProfileGroupMetadata,
    process: String,
    machine: String,
    filaments: Vec<String>,
}

impl ComposedProfile {
    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    pub fn into_settings(self) -> ProjectSettings {
        self.settings
    }

    pub fn metadata(&self) -> &ProfileGroupMetadata {
        &self.metadata
    }

    pub fn process_name(&self) -> &str {
        &self.process
    }

    pub fn machine_name(&self) -> &str {
        &self.machine
    }

    pub fn filament_names(&self) -> &[String] {
        &self.filaments
    }
}

pub fn compose_profile_fragments(
    fragments: &[ProfileFragment],
    selection: &ProfileSelection,
) -> Result<ComposedProfile, SliceError> {
    let machine = match selection.machine() {
        Some(name) => merge_profile_fragments(fragments, ProfileKind::Machine, name)?,
        None => unselected_profile(UnselectedKind::Machine),
    };
    let process = match selection.process() {
        Some(name) => merge_profile_fragments(fragments, ProfileKind::Process, name)?,
        None => unselected_profile(UnselectedKind::Process),
    };
    let filaments = selection
        .filaments()
        .iter()
        .map(|name| merge_profile_fragments(fragments, ProfileKind::Filament, name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(compose_resolved_profiles(
        selection,
        machine,
        process,
        filaments,
        ProjectRuntimeOptions::default(),
    ))
}

pub(super) fn compose_resolved_profiles(
    selection: &ProfileSelection,
    machine: MergedProfile,
    process: MergedProfile,
    filaments: Vec<MergedProfile>,
    runtime: ProjectRuntimeOptions,
) -> ComposedProfile {
    let MergedProfile::Machine {
        metadata: machine_metadata,
        options: printer,
    } = machine
    else {
        unreachable!("machine merge changed profile kind")
    };
    let MergedProfile::Process {
        metadata: process_metadata,
        options: process,
    } = process
    else {
        unreachable!("process merge changed profile kind")
    };

    let mut filament_metadata = Vec::with_capacity(filaments.len());
    let mut filament_options = Vec::with_capacity(filaments.len());
    for filament in filaments {
        let MergedProfile::Filament { metadata, options } = filament else {
            unreachable!("filament merge changed profile kind")
        };
        filament_metadata.push(metadata);
        filament_options.push(options);
    }

    let (filament, variant_cardinalities) = if filament_options.is_empty() {
        (crate::FilamentOptions::default(), Vec::new())
    } else {
        compose_filaments(filament_options)
    };
    let metadata = ProfileGroupMetadata::from_profiles(
        &process_metadata,
        &filament_metadata,
        &machine_metadata,
    );
    let project = project_options(
        selection,
        &process_metadata,
        &filament_metadata,
        variant_cardinalities,
        runtime,
    );

    ComposedProfile {
        settings: ProjectSettings {
            printer,
            process,
            filament,
            project,
            metadata: PresetMetadata::default(),
        },
        metadata,
        process: selection.process().unwrap_or_default().to_owned(),
        machine: selection.machine().unwrap_or_default().to_owned(),
        filaments: selection.filaments().to_vec(),
    }
}

fn project_options(
    selection: &ProfileSelection,
    process: &MergedProfileMetadata,
    filaments: &[MergedProfileMetadata],
    variant_cardinalities: Vec<usize>,
    mut project: ProjectRuntimeOptions,
) -> ProjectRuntimeOptions {
    project.preset.print_settings_id =
        OrcaString(selection.process().unwrap_or_default().to_owned());
    project.preset.printer_settings_id =
        OrcaString(selection.machine().unwrap_or_default().to_owned());
    project.preset.filament_settings_id = OrcaStrings(selection.filaments().to_vec());
    project.gcode.filament_map =
        OrcaInts(std::iter::repeat_n(OrcaInt(1), selection.filaments().len()).collect());
    project.gcode.filament_ids = OrcaStrings(
        filaments
            .iter()
            .map(|metadata| metadata.filament_id().unwrap_or_default().to_owned())
            .collect(),
    );
    if let Some(compatible_printers) = process
        .compatible_printers()
        .filter(|values| values.iter().any(|value| !value.is_empty()))
    {
        project.preset.print_compatible_printers = OrcaStrings(compatible_printers.to_vec());
    }
    project.preset.filament_self_index = OrcaInts(
        variant_cardinalities
            .into_iter()
            .enumerate()
            .flat_map(|(index, count)| std::iter::repeat_n(OrcaInt((index + 1) as i32), count))
            .collect(),
    );
    project
}
