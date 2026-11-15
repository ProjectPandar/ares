use std::collections::{BTreeMap, BTreeSet};

use super::fragment::{
    ProfileFragment, ProfileKind, metadata::ProfileConfigMetadataPatch, payload::ProfilePayload,
};
use crate::{
    FilamentOptions, PrinterOptions, ProcessOptions, SliceError,
    options::{PrinterOptionsBuilder, ProcessOptionsBuilder},
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
    /// A not-loaded load-settings kind: the upstream file loop runs its
    /// machine/process arms per loaded file only
    /// (`OrcaSlicer.cpp:2009-2106`), so a missing kind keeps the empty
    /// name the upstream variables retain.
    pub(crate) fn unselected(kind: ProfileKind) -> Self {
        Self {
            kind,
            name: String::new(),
            inherits: None,
            from: None,
            version: None,
            setting_id: None,
            instantiation: None,
            description: None,
            url: None,
            renamed_from: None,
            filament_id: None,
            compatible_printers: None,
            compatible_printers_condition: None,
            compatible_prints: None,
            compatible_prints_condition: None,
        }
    }
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

impl MergedProfile {
    pub(super) fn apply_system_metadata(&mut self, filament_id: Option<String>) {
        let metadata = match self {
            Self::Machine { metadata, .. }
            | Self::Process { metadata, .. }
            | Self::Filament { metadata, .. } => metadata,
        };
        // System loading consumes inherits as a loader key, not a config key
        // (PresetBundle.cpp:4862, Config.cpp:919-920). Filament IDs inherit only
        // when absent on the child (PresetBundle.cpp:4898-4910).
        metadata.inherits = None;
        if metadata.kind == ProfileKind::Filament {
            metadata.filament_id = filament_id;
        }
    }
}

/// A not-loaded load-settings kind: option defaults with unselected
/// metadata — a missing kind contributes no overlay fields, matching the
/// per-loaded-file arms of `OrcaSlicer.cpp:2009-2106`. Only machine and
/// process can be partial; filament lists are never partial in the
/// load-settings entry (a settings-only list stays rejected until its own
/// RED covers it), so the kind set is closed here.
#[derive(Clone, Copy)]
pub enum UnselectedKind {
    Machine,
    Process,
}

pub fn unselected_profile(kind: UnselectedKind) -> MergedProfile {
    let metadata = MergedProfileMetadata::unselected(match kind {
        UnselectedKind::Machine => ProfileKind::Machine,
        UnselectedKind::Process => ProfileKind::Process,
    });
    match kind {
        UnselectedKind::Machine => MergedProfile::Machine {
            metadata,
            options: PrinterOptionsBuilder::default().resolve(),
        },
        UnselectedKind::Process => MergedProfile::Process {
            metadata,
            options: ProcessOptionsBuilder::default().resolve(),
        },
    }
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
    merge_chain(&chain, target_kind)
}

/// Single-record composition for sparse typed CLI files: the early return
/// at `OrcaSlicer.cpp:2121-2123` bypasses preset resolution without inherits.
/// Parent metadata follows CLI file-source rules:
/// system machine/process parents are cleared (`:2838-2842,2981-2985`),
/// and filament parents are retained only for `User`/`user` (`:2189-2193`).
pub(super) fn merge_direct_profile_fragment(
    fragments: &[ProfileFragment],
    target_kind: ProfileKind,
    target_name: &str,
) -> Result<MergedProfile, SliceError> {
    let index = build_index(fragments)?;
    let selected = *index
        .get(&(target_kind, target_name.to_owned()))
        .ok_or_else(|| {
            SliceError::InvalidInput(format!("profile '{target_name}' was not found"))
        })?;
    let mut merged = merge_chain(&[&fragments[selected]], target_kind)?;
    let metadata = match &mut merged {
        MergedProfile::Machine { metadata, .. }
        | MergedProfile::Process { metadata, .. }
        | MergedProfile::Filament { metadata, .. } => metadata,
    };
    let keep_parent = match target_kind {
        ProfileKind::Machine | ProfileKind::Process => metadata.from() != Some("system"),
        ProfileKind::Filament => matches!(metadata.from(), Some("User" | "user")),
    };
    if !keep_parent {
        metadata.inherits = None;
    }
    Ok(merged)
}

fn merge_chain(
    chain: &[&ProfileFragment],
    target_kind: ProfileKind,
) -> Result<MergedProfile, SliceError> {
    let selected = chain[chain.len() - 1];
    let metadata = merge_metadata(chain, selected);

    match target_kind {
        ProfileKind::Machine => {
            let mut merged = PrinterOptionsBuilder::default();
            for fragment in chain {
                let ProfilePayload::Machine(child) = fragment.payload() else {
                    unreachable!("same-kind profile chain changed payload owner")
                };
                merged.overlay(child.as_ref().clone());
            }
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
            let mut chain = chain.iter();
            let ProfilePayload::Filament(root) = chain
                .next()
                .expect("collected profile chain contains its selected root")
                .payload()
            else {
                unreachable!("same-kind profile chain changed payload owner")
            };
            let mut merged = root.as_ref().clone().resolve_profile_root()?;
            for fragment in chain {
                let ProfilePayload::Filament(child) = fragment.payload() else {
                    unreachable!("same-kind profile chain changed payload owner")
                };
                child.as_ref().clone().apply_profile_child(&mut merged)?;
            }
            Ok(MergedProfile::Filament {
                metadata,
                options: merged,
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
