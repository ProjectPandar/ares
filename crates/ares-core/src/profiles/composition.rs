mod filament;
mod metadata;

use super::{
    fragment::{ProfileFragment, ProfileKind, merge_profile_fragments},
    inheritance::{MergedProfile, MergedProfileMetadata},
};
use crate::{
    OrcaInt, OrcaInts, OrcaString, OrcaStrings, PresetMetadata, ProjectRuntimeOptions,
    ProjectSettings, SliceError,
};

use self::filament::compose_filaments;
pub use self::metadata::ProfileGroupMetadata;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileSelection {
    process: String,
    machine: String,
    filaments: Vec<String>,
}

impl ProfileSelection {
    pub fn new(
        process: impl Into<String>,
        machine: impl Into<String>,
        filaments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SliceError> {
        let process = process.into();
        let machine = machine.into();
        let filaments = filaments.into_iter().map(Into::into).collect::<Vec<_>>();
        if process.is_empty()
            || machine.is_empty()
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

    pub fn process(&self) -> &str {
        &self.process
    }

    pub fn machine(&self) -> &str {
        &self.machine
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
    let MergedProfile::Machine {
        metadata: machine_metadata,
        options: printer,
    } = merge_profile_fragments(fragments, ProfileKind::Machine, selection.machine())?
    else {
        unreachable!("machine merge changed profile kind")
    };
    let MergedProfile::Process {
        metadata: process_metadata,
        options: process,
    } = merge_profile_fragments(fragments, ProfileKind::Process, selection.process())?
    else {
        unreachable!("process merge changed profile kind")
    };

    let mut filament_metadata = Vec::with_capacity(selection.filaments().len());
    let mut filament_options = Vec::with_capacity(selection.filaments().len());
    for name in selection.filaments() {
        let MergedProfile::Filament { metadata, options } =
            merge_profile_fragments(fragments, ProfileKind::Filament, name)?
        else {
            unreachable!("filament merge changed profile kind")
        };
        filament_metadata.push(metadata);
        filament_options.push(options);
    }

    let (filament, variant_cardinalities) = compose_filaments(filament_options);
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
    );

    Ok(ComposedProfile {
        settings: ProjectSettings {
            printer,
            process,
            filament,
            project,
            metadata: PresetMetadata::default(),
        },
        metadata,
        process: selection.process().to_owned(),
        machine: selection.machine().to_owned(),
        filaments: selection.filaments().to_vec(),
    })
}

fn project_options(
    selection: &ProfileSelection,
    process: &MergedProfileMetadata,
    filaments: &[MergedProfileMetadata],
    variant_cardinalities: Vec<usize>,
) -> ProjectRuntimeOptions {
    let mut project = ProjectRuntimeOptions::default();
    project.preset.print_settings_id = OrcaString(selection.process().to_owned());
    project.preset.printer_settings_id = OrcaString(selection.machine().to_owned());
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
