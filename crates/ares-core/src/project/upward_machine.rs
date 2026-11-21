//! OrcaSlicer.cpp:1732-1769,2517-2536: original machine, not final overlays.
use serde::Deserialize;

use super::{
    ArchiveLimits, PackagePath, ProjectArchive,
    raw_settings::ProjectSettingsRaw,
    relationships::{MODEL_RELATIONSHIP_TYPE, Relationships},
    xml::{XmlRole, deserialize_xml},
};
use crate::SliceError;

/// Original BBL machine identity and its embedded upward vector. Adapters
/// supply optional machine_full JSON bytes; core never resolves filesystem paths.
pub struct UpwardMachineSource {
    system_printer: String,
    system_process: String,
    system_filaments: Vec<String>,
    embedded: Vec<String>,
}

/// Mutually exclusive upstream source branches. Public CLI refresh/overlay
/// orchestration is separate; these inputs port only their vector selection.
pub enum UpwardMachineInput<'a> {
    Embedded,
    /// Process-source name checks are owned by the refresh controller,
    /// not this upward-vector projection. That public controller is deferred.
    UpToDateFiles(&'a [crate::LoadSettingsFile<'a>]),
    UpToDateOriginal(Option<&'a [u8]>),
    RetainedRefresh(&'a crate::NativeRefreshPresets),
    FetchOriginal(Option<&'a [u8]>),
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct MachineFields {
    printer_settings_id: String,
    print_settings_id: String,
    filament_settings_id: Vec<String>,
    inherits_group: Option<Vec<String>>,
    upward_compatible_machine: Vec<String>,
}

#[derive(Deserialize)]
struct ModelIdentity {
    #[serde(rename = "$value", default)]
    children: Vec<IdentityChild>,
}

#[derive(Deserialize)]
enum IdentityChild {
    #[serde(rename = "metadata")]
    Metadata(Metadata),
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "$text", default)]
    value: String,
}

impl UpwardMachineSource {
    /// Reads only package identity/settings, without building or copying mesh
    /// geometry. Non-BBL projects have no original BBL machine fetch.
    pub fn from_project(input: &[u8]) -> Result<Option<Self>, SliceError> {
        let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
        let root = PackagePath::root();
        let relationships: Relationships = deserialize_xml(
            &archive.read(&root.resolve("_rels/.rels")?)?,
            XmlRole::Relationships,
        )?;
        let model_path = relationships.resolve_required(&root, MODEL_RELATIONSHIP_TYPE)?;
        let identity: ModelIdentity = deserialize_xml(&archive.read(&model_path)?, XmlRole::Model)?;
        if !identity.children.iter().any(|child| {
            let IdentityChild::Metadata(entry) = child else {
                return false;
            };
            entry.name == "Application"
                && (entry.value.starts_with("BambuStudio-")
                    || entry.value.starts_with("OrcaSlicer-"))
        }) {
            return Ok(None);
        }
        let fields = fields(&archive.read(&root.resolve("Metadata/project_settings.config")?)?)?;
        let group = fields
            .inherits_group
            .as_deref()
            .filter(|group| group.len() >= 2)
            .unwrap_or_default();
        let system_name = |identity, inherited: Option<&String>| {
            inherited
                .filter(|name| !name.is_empty())
                .cloned()
                .unwrap_or(identity)
        };
        let system_printer = system_name(fields.printer_settings_id, group.last());
        let system_process = system_name(fields.print_settings_id, group.first());
        let system_filaments = fields
            .filament_settings_id
            .into_iter()
            .enumerate()
            .map(|(slot, name)| {
                system_name(
                    name,
                    group
                        .get(1..group.len().saturating_sub(1))
                        .and_then(|group| group.get(slot)),
                )
            })
            .collect();
        Ok(Some(Self {
            system_printer,
            system_process,
            system_filaments,
            embedded: fields.upward_compatible_machine,
        }))
    }

    pub fn system_printer(&self) -> &str {
        &self.system_printer
    }

    pub fn system_process(&self) -> &str {
        &self.system_process
    }

    pub fn system_filaments(&self) -> &[String] {
        &self.system_filaments
    }

    /// CLI:1769/2301/2385/2534. Missing original-machine files keep the
    /// embedded vector; present files replace it, even with an empty vector.
    pub fn select(self, input: UpwardMachineInput<'_>) -> Result<Vec<String>, SliceError> {
        match input {
            UpwardMachineInput::Embedded => Ok(self.embedded),
            UpwardMachineInput::UpToDateFiles(files) => self.explicit_values(files),
            UpwardMachineInput::RetainedRefresh(presets) => Ok(presets
                .machine
                .as_ref()
                .map_or(self.embedded, |preset| preset.upward_machines())),
            UpwardMachineInput::UpToDateOriginal(machine_full)
            | UpwardMachineInput::FetchOriginal(machine_full) => {
                if self.system_printer.is_empty() {
                    return Ok(self.embedded);
                }
                match machine_full {
                    Some(json) => {
                        let profile =
                            crate::profiles::ProfileFragment::from_original_machine_bytes(json)?;
                        validate_source(&profile)?;
                        Ok(fields(json)?.upward_compatible_machine)
                    }
                    None => Ok(self.embedded),
                }
            }
        }
    }
    fn explicit_values(
        self,
        files: &[crate::LoadSettingsFile<'_>],
    ) -> Result<Vec<String>, SliceError> {
        let mut values = self.embedded;
        for file in files {
            let profile = file_profile(file.json.as_bytes())?;
            match profile.kind() {
                crate::profiles::ProfileKind::Machine => {}
                // Process update/name validation belongs to its controller.
                crate::profiles::ProfileKind::Process => continue,
                crate::profiles::ProfileKind::Filament => {
                    return Err(SliceError::InvalidInput(
                        "invalid up-to-date config type".into(),
                    ));
                }
            }
            if profile.name() != self.system_printer {
                return Err(SliceError::InvalidInput(
                    "up-to-date machine does not match original system printer".into(),
                ));
            }
            values = fields(file.json.as_bytes())?.upward_compatible_machine;
        }
        Ok(values)
    }
}

// OrcaSlicer.cpp:2205–2263 accepts system, User and user sources. A failed
// explicit-file load must not be interpreted as an absent machine file.
fn file_profile(json: &[u8]) -> Result<crate::profiles::ProfileFragment, SliceError> {
    let profile = crate::profiles::ProfileFragment::from_up_to_date_file_bytes(json)?;
    validate_source(&profile)?;
    Ok(profile)
}

fn validate_source(profile: &crate::profiles::ProfileFragment) -> Result<(), SliceError> {
    if !matches!(profile.from(), Some("system" | "User" | "user")) {
        return Err(SliceError::InvalidInput(
            "unsupported machine source from metadata".into(),
        ));
    }
    Ok(())
}

fn fields(json: &[u8]) -> Result<MachineFields, SliceError> {
    let raw: ProjectSettingsRaw = serde_json::from_slice(crate::profiles::without_utf8_bom(json))
        .map_err(|error| {
        SliceError::InvalidInput(format!("invalid machine source JSON: {error}"))
    })?;
    let mut selected = ProjectSettingsRaw::default();
    selected.extend(
        raw.entries()
            .filter(|(key, _)| {
                matches!(
                    key.as_str(),
                    "printer_settings_id"
                        | "print_settings_id"
                        | "filament_settings_id"
                        | "inherits_group"
                        | "upward_compatible_machine"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    selected
        .normalize_option_scalars()
        .map_err(SliceError::InvalidInput)?;
    let bytes = serde_json::to_vec(&selected)
        .map_err(|error| SliceError::InvalidInput(error.to_string()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| SliceError::InvalidInput(format!("invalid machine source: {error}")))
}
