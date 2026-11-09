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
    FetchOriginal(Option<&'a [u8]>),
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct MachineFields {
    printer_settings_id: String,
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
        let system_printer = match fields.inherits_group {
            Some(mut group) => {
                let last = group.pop().ok_or_else(|| {
                    SliceError::InvalidInput("empty printer inherits_group".into())
                })?;
                if last.is_empty() {
                    fields.printer_settings_id
                } else {
                    last
                }
            }
            None => fields.printer_settings_id,
        };
        Ok(Some(Self {
            system_printer,
            embedded: fields.upward_compatible_machine,
        }))
    }

    pub fn system_printer(&self) -> &str {
        &self.system_printer
    }

    /// CLI:1769/2301/2385/2534. Missing original-machine files keep the
    /// embedded vector; present files replace it, even with an empty vector.
    pub fn select(self, input: UpwardMachineInput<'_>) -> Result<Vec<String>, SliceError> {
        match input {
            UpwardMachineInput::Embedded => Ok(self.embedded),
            UpwardMachineInput::UpToDateFiles(files) => self.explicit_values(files),
            UpwardMachineInput::UpToDateOriginal(machine_full)
            | UpwardMachineInput::FetchOriginal(machine_full) => {
                if self.system_printer.is_empty() {
                    return Ok(self.embedded);
                }
                match machine_full {
                    Some(json) => {
                        file_profile(json)?;
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

// CLI:1953-2017 accepts system, User and user sources. A failed load must
// not be interpreted as an absent original-machine file.
fn file_profile(json: &[u8]) -> Result<crate::profiles::ProfileFragment, SliceError> {
    let profile = crate::profiles::ProfileFragment::from_json_bytes(json)?;
    if !matches!(profile.from(), Some("system" | "User" | "user")) {
        return Err(SliceError::InvalidInput(
            "unsupported machine source from metadata".into(),
        ));
    }
    Ok(profile)
}

fn fields(json: &[u8]) -> Result<MachineFields, SliceError> {
    let raw: ProjectSettingsRaw = serde_json::from_slice(json).map_err(|error| {
        SliceError::InvalidInput(format!("invalid machine source JSON: {error}"))
    })?;
    let mut selected = ProjectSettingsRaw::default();
    selected.extend(
        raw.entries()
            .filter(|(key, _)| {
                matches!(
                    key.as_str(),
                    "printer_settings_id" | "inherits_group" | "upward_compatible_machine"
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
