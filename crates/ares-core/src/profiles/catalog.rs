mod cli_metadata;
mod document;
mod raw;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{
    ComposedProfile, MergedProfile, ProfileFragment, ProfileKind, ProfileSelection,
    composition::compose_resolved_profiles, merge_profile_fragments,
};
use crate::{
    SliceError, options::ProjectRuntimeOptionsBuilder, project::raw_settings::ProjectSettingsRaw,
};

use document::{CatalogInput, ProfileDocument, VendorInput};

pub(super) struct Catalog {
    records: Vec<Record>,
    index: BTreeMap<(usize, ProfileKind, String), usize>,
    selectable: BTreeMap<(ProfileKind, String), usize>,
    library: Option<usize>,
    user_scope: usize,
}

struct Record {
    scope: usize,
    kind: ProfileKind,
    document: ProfileDocument,
}

impl Catalog {
    pub(super) fn from_json(input: &str) -> Result<Self, SliceError> {
        let input: CatalogInput = serde_json::from_str(input).map_err(invalid_json)?;
        let user_scope = input.vendors.len();
        let mut catalog = Self {
            records: Vec::new(),
            index: BTreeMap::new(),
            selectable: BTreeMap::new(),
            library: None,
            user_scope,
        };
        for (scope, vendor) in input.vendors.into_iter().enumerate() {
            catalog.insert_vendor(scope, vendor)?;
        }
        for package in input.imports {
            for document in package.profiles {
                let kind = ProfileKind::parse(&document.kind)?;
                catalog.insert(user_scope, kind, document)?;
            }
        }
        Ok(catalog)
    }

    fn insert_vendor(&mut self, scope: usize, mut vendor: VendorInput) -> Result<(), SliceError> {
        if vendor.index.name == "OrcaFilamentLibrary" {
            self.library = Some(scope);
        }
        let entries = [
            (ProfileKind::Machine, vendor.index.machine_list),
            (ProfileKind::Process, vendor.index.process_list),
            (ProfileKind::Filament, vendor.index.filament_list),
        ]
        .into_iter()
        .flat_map(|(kind, entries)| entries.into_iter().map(move |entry| (kind, entry)));
        for (kind, entry) in entries {
            let document = vendor.files.remove(&entry.sub_path).ok_or_else(|| {
                invalid(format!("missing vendor profile file {}", entry.sub_path))
            })?;
            self.insert(scope, kind, document)?;
        }
        Ok(())
    }

    fn insert(
        &mut self,
        scope: usize,
        kind: ProfileKind,
        document: ProfileDocument,
    ) -> Result<(), SliceError> {
        let position = self.records.len();
        let key = (kind, document.name.clone());
        if self
            .index
            .insert((scope, kind, document.name.clone()), position)
            .is_some()
        {
            return Err(invalid(format!(
                "duplicate profile {} within vendor scope",
                document.name
            )));
        }
        if document.instantiation.as_deref() != Some("false")
            && self.selectable.insert(key, position).is_some()
        {
            return Err(invalid(format!(
                "ambiguous selectable profile {}",
                document.name
            )));
        }
        self.records.push(Record {
            scope,
            kind,
            document,
        });
        Ok(())
    }

    pub(super) fn compose(
        &self,
        selection: &ProfileSelection,
    ) -> Result<(ComposedProfile, ProjectSettingsRaw), SliceError> {
        let mut raw = ProjectSettingsRaw::default();
        let mut runtime = ProjectRuntimeOptionsBuilder::default();
        // OrcaSlicer.cpp:2849-2867,2993-3011 applies machine, then process.
        let machine = self.merge(
            ProfileKind::Machine,
            selection.machine(),
            &mut raw,
            &mut runtime,
        )?;
        let process = self.merge(
            ProfileKind::Process,
            selection.process(),
            &mut raw,
            &mut runtime,
        )?;
        let filaments = selection
            .filaments()
            .iter()
            .map(|name| self.merge(ProfileKind::Filament, name, &mut raw, &mut runtime))
            .collect::<Result<Vec<_>, _>>()?;
        cli_metadata::filament_conditions(&mut raw, &filaments);
        let composed =
            compose_resolved_profiles(selection, machine, process, filaments, runtime.resolve());
        if let Some(group) = composed.metadata().inherits_group() {
            raw.insert_strings("inherits_group", group);
        }
        Ok((composed, raw))
    }

    fn merge(
        &self,
        kind: ProfileKind,
        name: &str,
        raw: &mut ProjectSettingsRaw,
        runtime: &mut ProjectRuntimeOptionsBuilder,
    ) -> Result<MergedProfile, SliceError> {
        let position = *self
            .selectable
            .get(&(kind, name.to_owned()))
            .ok_or_else(|| invalid(format!("selectable profile '{name}' was not found")))?;
        let mut chain = Vec::new();
        self.collect(position, &mut BTreeSet::new(), &mut chain)?;
        let mut fragments = Vec::with_capacity(chain.len());
        for record in chain {
            let (fragment, project) =
                ProfileFragment::from_catalog_values(record.document.values())?;
            fragments.push(fragment);
            runtime.overlay(project);
            raw.extend(
                record
                    .document
                    .fields
                    .iter()
                    .filter(|(key, _)| raw::applies(kind, key))
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        let mut merged = merge_profile_fragments(&fragments, kind, name)?;
        if self.records[position].scope != self.user_scope {
            let filament_id = fragments
                .iter()
                .rev()
                .filter_map(ProfileFragment::filament_id)
                .find(|value| !value.is_empty())
                .map(str::to_owned);
            merged.apply_system_metadata(filament_id);
        }
        Ok(merged)
    }

    fn collect<'a>(
        &'a self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
        chain: &mut Vec<&'a Record>,
    ) -> Result<(), SliceError> {
        if !visiting.insert(position) {
            return Err(invalid("profile inheritance cycle"));
        }
        let record = &self.records[position];
        if let Some(parent) = record
            .document
            .inherits
            .as_deref()
            .filter(|name| !name.is_empty())
        {
            let parent_position = self.parent_position(record, parent).ok_or_else(|| {
                invalid(format!(
                    "parent profile '{parent}' was not found for '{}'",
                    record.document.name
                ))
            })?;
            self.collect(parent_position, visiting, chain)?;
        }
        chain.push(record);
        Ok(())
    }

    fn parent_position(&self, record: &Record, parent: &str) -> Option<usize> {
        // PresetBundle.cpp:4885-4918: local templates precede the base library.
        let key = (record.scope, record.kind, parent.to_owned());
        if let Some(position) = self.index.get(&key) {
            return Some(*position);
        }
        if record.scope == self.user_scope {
            return self
                .selectable
                .get(&(record.kind, parent.to_owned()))
                .copied();
        }
        self.library.and_then(|scope| {
            self.index
                .get(&(scope, record.kind, parent.to_owned()))
                .copied()
        })
    }
}

#[derive(Deserialize)]
struct SelectionInput {
    machine: String,
    process: String,
    filaments: Vec<String>,
}

pub(super) fn selection(input: &str) -> Result<ProfileSelection, SliceError> {
    let input: SelectionInput = serde_json::from_str(input).map_err(invalid_json)?;
    ProfileSelection::new(input.process, input.machine, input.filaments)
}

fn invalid_json(error: serde_json::Error) -> SliceError {
    invalid(format!(
        "invalid profile catalog or selection JSON: {error}"
    ))
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
