use std::collections::{BTreeMap, BTreeSet};

use super::fragment::{
    ProfileFragment, ProfileKind, metadata::ProfileConfigMetadataPatch, payload::ProfilePayload,
};
use crate::{
    FilamentOptions, PrinterOptions, ProcessOptions, SliceError,
    options::{FilamentOptionsBuilder, PrinterOptionsBuilder, ProcessOptionsBuilder},
};

#[derive(Clone, Debug, PartialEq)]
pub struct MergedProfileMetadata {
    kind: ProfileKind,
    name: String,
    inherits: Option<String>,
    from: Option<String>,
    version: Option<String>,
    setting_id: Option<String>,
    instantiation: Option<String>,
    description: Option<String>,
    url: Option<String>,
    renamed_from: Option<String>,
    filament_id: Option<String>,
    compatible_printers: Option<Vec<String>>,
    compatible_printers_condition: Option<String>,
    compatible_prints: Option<Vec<String>>,
    compatible_prints_condition: Option<String>,
}

impl MergedProfileMetadata {
    pub const fn kind(&self) -> ProfileKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn inherits(&self) -> Option<&str> {
        self.inherits.as_deref()
    }

    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn setting_id(&self) -> Option<&str> {
        self.setting_id.as_deref()
    }

    pub fn instantiation(&self) -> Option<&str> {
        self.instantiation.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn renamed_from(&self) -> Option<&str> {
        self.renamed_from.as_deref()
    }

    pub fn filament_id(&self) -> Option<&str> {
        self.filament_id.as_deref()
    }

    pub fn compatible_printers(&self) -> Option<&[String]> {
        self.compatible_printers.as_deref()
    }

    pub fn compatible_printers_condition(&self) -> Option<&str> {
        self.compatible_printers_condition.as_deref()
    }

    pub fn compatible_prints(&self) -> Option<&[String]> {
        self.compatible_prints.as_deref()
    }

    pub fn compatible_prints_condition(&self) -> Option<&str> {
        self.compatible_prints_condition.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum MergedProfile {
    Machine {
        metadata: MergedProfileMetadata,
        options: PrinterOptions,
    },
    Process {
        metadata: MergedProfileMetadata,
        options: ProcessOptions,
    },
    Filament {
        metadata: MergedProfileMetadata,
        options: FilamentOptions,
    },
}

pub fn merge_profile_fragments(
    fragments: &[ProfileFragment],
    target_kind: ProfileKind,
    target_name: &str,
) -> Result<MergedProfile, SliceError> {
    let resolver = ChainResolver {
        fragments,
        index: build_index(fragments)?,
    };
    let mut chain = Vec::new();
    resolver.collect_chain(target_kind, target_name, &mut BTreeSet::new(), &mut chain)?;
    let selected = chain[chain.len() - 1];
    let metadata = merge_metadata(&chain, selected);

    match target_kind {
        ProfileKind::Machine => {
            let mut merged = PrinterOptionsBuilder::default();
            for fragment in chain {
                let ProfilePayload::Machine(child) = fragment.payload() else {
                    unreachable!("same-kind profile chain changed payload owner")
                };
                merged.overlay(child.as_ref().clone());
            }
            merged.normalize_present_thumbnails().map_err(|error| {
                SliceError::InvalidInput(format!(
                    "profile option thumbnails is invalid: {}",
                    crate::thumbnail_error_string(error)
                ))
            })?;
            Ok(MergedProfile::Machine {
                metadata,
                options: merged.resolve(),
            })
        }
        ProfileKind::Process => {
            let mut merged = ProcessOptionsBuilder::default();
            for fragment in chain {
                let ProfilePayload::Process(child) = fragment.payload() else {
                    unreachable!("same-kind profile chain changed payload owner")
                };
                merged.overlay(child.as_ref().clone());
            }
            Ok(MergedProfile::Process {
                metadata,
                options: merged.resolve(),
            })
        }
        ProfileKind::Filament => {
            let mut merged = FilamentOptionsBuilder::default();
            for fragment in chain {
                let ProfilePayload::Filament(child) = fragment.payload() else {
                    unreachable!("same-kind profile chain changed payload owner")
                };
                merged.overlay(child.as_ref().clone());
            }
            Ok(MergedProfile::Filament {
                metadata,
                options: merged.resolve(),
            })
        }
    }
}

fn build_index(
    fragments: &[ProfileFragment],
) -> Result<BTreeMap<(ProfileKind, String), usize>, SliceError> {
    let mut index = BTreeMap::new();
    for (position, fragment) in fragments.iter().enumerate() {
        let key = (fragment.kind(), fragment.name().to_owned());
        if index.insert(key, position).is_some() {
            return Err(SliceError::InvalidInput(
                "duplicate profile fragment".to_owned(),
            ));
        }
    }
    Ok(index)
}

struct ChainResolver<'a> {
    fragments: &'a [ProfileFragment],
    index: BTreeMap<(ProfileKind, String), usize>,
}

impl<'a> ChainResolver<'a> {
    fn collect_chain(
        &self,
        kind: ProfileKind,
        name: &str,
        visiting: &mut BTreeSet<(ProfileKind, String)>,
        chain: &mut Vec<&'a ProfileFragment>,
    ) -> Result<(), SliceError> {
        let key = (kind, name.to_owned());
        if !visiting.insert(key.clone()) {
            return Err(SliceError::InvalidInput(
                "profile inheritance cycle".to_owned(),
            ));
        }
        let Some(position) = self.index.get(&key).copied() else {
            let detail = if self.index.keys().any(|(_, candidate)| candidate == name) {
                "has a different profile kind"
            } else {
                "was not found"
            };
            return Err(SliceError::InvalidInput(format!(
                "profile '{name}' {detail}"
            )));
        };
        let fragment = &self.fragments[position];
        if let Some(parent) = fragment.inherits() {
            self.collect_chain(kind, parent, visiting, chain)?;
        }
        chain.push(fragment);
        Ok(())
    }
}

fn merge_metadata(chain: &[&ProfileFragment], selected: &ProfileFragment) -> MergedProfileMetadata {
    let mut config = ProfileConfigMetadataPatch::default();
    for fragment in chain {
        config.overlay_compatibility(fragment.config());
    }
    let filament_id = if selected.kind() == ProfileKind::Filament && selected.inherits().is_some() {
        chain[0].filament_id().map(str::to_owned)
    } else {
        selected.filament_id().map(str::to_owned)
    };
    MergedProfileMetadata {
        kind: selected.kind(),
        name: selected.name().to_owned(),
        inherits: selected.inherits().map(str::to_owned),
        from: selected.from().map(str::to_owned),
        version: selected.version().map(str::to_owned),
        setting_id: selected.setting_id().map(str::to_owned),
        instantiation: selected.instantiation().map(str::to_owned),
        description: selected.description().map(str::to_owned),
        url: selected.url().map(str::to_owned),
        renamed_from: selected.renamed_from().map(str::to_owned),
        filament_id,
        compatible_printers: config.compatible_printers,
        compatible_printers_condition: config.compatible_printers_condition,
        compatible_prints: config.compatible_prints,
        compatible_prints_condition: config.compatible_prints_condition,
    }
}
