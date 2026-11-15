use serde::Serialize;

use super::{
    super::{catalog::Catalog, project_refresh::preset::connection},
    config::PresetConfig,
};
use crate::{
    ProfileKind, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

#[derive(Clone, Debug, Serialize)]
pub struct PresetRecord {
    pub name: String,
    pub version: String,
    pub default: bool,
    pub system: bool,
    pub external: bool,
    pub project_local: bool,
    pub visible: bool,
    pub dirty: bool,
    pub filament_id: String,
    pub config: PresetConfig,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetOutcome {
    ExistingClean,
    ExistingEdited,
    ProjectLocal,
}

#[derive(Clone, Debug, Serialize)]
pub struct PresetCollectionState {
    pub outcome: PresetOutcome,
    pub selected: PresetRecord,
    pub edited: PresetRecord,
    pub dirty: bool,
    pub dirty_options: Vec<String>,
    pub records: Vec<PresetRecord>,
}

pub(super) struct Collection {
    pub(super) kind: ProfileKind,
    pub(super) records: Vec<PresetRecord>,
    renamed: Vec<(String, usize)>,
}

impl Collection {
    pub(super) fn load(catalog: &Catalog, kind: ProfileKind) -> Result<Self, SliceError> {
        let mut records = vec![PresetRecord {
            name: match kind {
                ProfileKind::Process => "Default Setting",
                ProfileKind::Machine => "Default Printer",
                ProfileKind::Filament => "Default Filament",
            }
            .to_owned(),
            version: "0.0.0.0".to_owned(),
            default: true,
            system: false,
            external: false,
            project_local: false,
            visible: true,
            dirty: false,
            filament_id: String::new(),
            config: PresetConfig::defaults(kind)?,
        }];
        let mut aliases = Vec::new();
        for position in catalog.selectable_positions(kind) {
            if catalog.missing_parent(position).is_some() {
                continue;
            }
            let system = !catalog.is_user_scope(position);
            let document = catalog.document(position);
            let (config, filament_id) = load_config(catalog, kind, position)?;
            if system && let Some(value) = document.fields.get("renamed_from") {
                aliases.extend(
                    crate::options::unescape_strings_cstyle(&value.token().unwrap_or_default())
                        .map_err(SliceError::InvalidInput)?
                        .into_iter()
                        .map(|old| (old, document.name.clone())),
                );
            }
            records.push(PresetRecord {
                name: document.name.clone(),
                version: version(
                    document
                        .fields
                        .get("version")
                        .and_then(RawConfigValue::token)
                        .as_deref()
                        .unwrap_or("0.0.0.0"),
                )?,
                default: false,
                system,
                external: false,
                project_local: false,
                visible: kind != ProfileKind::Machine || !system,
                dirty: false,
                filament_id,
                config,
            });
        }
        records[1..].sort_by(|left, right| left.name.cmp(&right.name));
        records[0].visible = !records[1..].iter().any(|record| record.visible);
        let renamed = aliases
            .into_iter()
            .map(|(old, name)| {
                (
                    old,
                    records
                        .iter()
                        .position(|record| record.name == name)
                        .expect("loaded renamed system preset"),
                )
            })
            .collect();
        Ok(Self {
            kind,
            records,
            renamed,
        })
    }

    pub(super) fn materialize(
        &self,
        project: &ProjectSettingsRaw,
        protected: &std::collections::BTreeSet<String>,
    ) -> Result<PresetConfig, SliceError> {
        let mut config = PresetConfig::defaults(self.kind)?;
        for (key, value) in project.entries() {
            if config.0.contains_key(key) && !connection(key) {
                config.0.insert(key.clone(), value.clone());
            }
        }
        let parent = config.string("inherits");
        let base = if parent.is_empty() {
            self.find(&config.string(identity(self.kind)))
                .filter(|index| self.records[*index].system)
        } else {
            self.find(&parent)
        };
        if !protected.is_empty()
            && let Some(index) = base
        {
            super::variants::refresh(
                &mut config,
                &self.records[index].config,
                protected,
                self.kind,
            )?;
        }
        Ok(config)
    }

    pub(super) fn find(&self, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        self.exact(name).or_else(|| {
            self.renamed
                .iter()
                .find(|(old, _)| old == name)
                .map(|(_, index)| *index)
        })
    }

    fn exact(&self, name: &str) -> Option<usize> {
        self.records.iter().position(|record| record.name == name)
    }

    // Preset.cpp:3027-3262, LoadAndSelect::Always: installed reference is never replaced by its edit.
    pub(super) fn select(
        mut self,
        mut config: PresetConfig,
        filename: &str,
        project_version: &str,
        filament_id: &str,
    ) -> Result<PresetCollectionState, SliceError> {
        let identity = identity(self.kind);
        let original = config.string(identity);
        let parent = config.string("inherits");
        if let Some(index) = self.find(&original).or_else(|| self.find(&parent)) {
            if self.records[index].config.same(&config)? {
                return self.finish(index, PresetOutcome::ExistingClean, None);
            }
            let mut edited = self.records[index].config.clone();
            for (key, value) in config.0.entries().filter(|(key, _)| !connection(key)) {
                edited.0.insert(key.clone(), value.clone());
            }
            return self.finish(index, PresetOutcome::ExistingEdited, Some(edited));
        }
        let prefix = if original.is_empty() {
            &parent
        } else {
            &original
        };
        let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
        let mut suffix = 0;
        loop {
            let number = if suffix == 0 {
                String::new()
            } else if prefix.is_empty() {
                suffix.to_string()
            } else {
                format!("-{suffix}")
            };
            let name = format!("{prefix}{number}({basename})");
            let found = self.exact(&name);
            if let Some(index) = found
                && self.records[index].config.same(&config)?
            {
                return self.finish(index, PresetOutcome::ExistingClean, None);
            }
            if found.is_some() {
                suffix += 1;
                continue;
            }
            config.set_string(identity, name.clone());
            self.records.push(PresetRecord {
                name,
                version: version(project_version)?,
                default: false,
                system: false,
                external: true,
                project_local: true,
                visible: true,
                dirty: false,
                filament_id: filament_id.to_owned(),
                config,
            });
            self.records[1..].sort_by(|left, right| left.name.cmp(&right.name));
            let index = self
                .records
                .iter()
                .position(|record| record.project_local)
                .expect("inserted project preset");
            return self.finish(index, PresetOutcome::ProjectLocal, None);
        }
    }

    fn finish(
        mut self,
        index: usize,
        outcome: PresetOutcome,
        edited_config: Option<PresetConfig>,
    ) -> Result<PresetCollectionState, SliceError> {
        for record in &mut self.records {
            record.dirty = false;
        }
        let mut edited = self.records[index].clone();
        self.records[0].visible = index == 0;
        self.records[index].visible = true;
        if let Some(config) = edited_config {
            edited.config = config;
        }
        let (dirty, dirty_options) = self.records[index].config.dirty(&edited.config)?;
        edited.dirty = dirty;
        self.records[index].dirty = dirty;
        Ok(PresetCollectionState {
            outcome,
            selected: self.records[index].clone(),
            edited,
            dirty,
            dirty_options,
            records: self.records,
        })
    }
}

fn load_config(
    catalog: &Catalog,
    kind: ProfileKind,
    position: usize,
) -> Result<(PresetConfig, String), SliceError> {
    let system = !catalog.is_user_scope(position);
    let mut config = PresetConfig::defaults(kind)?;
    let mut filament_id = String::new();
    for layer in catalog.resolved_layers(position)? {
        let mut raw = ProjectSettingsRaw::default();
        raw.extend(layer.fields.clone());
        raw.handle_legacy();
        for (key, value) in raw.entries() {
            if config.0.contains_key(key) {
                config.0.insert(key.clone(), value.clone());
            }
        }
        if let Some(value) = layer.fields.get("filament_id") {
            filament_id = value.token().unwrap_or_default();
        }
    }
    config.set_string(
        "inherits",
        if system {
            String::new()
        } else {
            catalog.leaf_inherits(position).unwrap_or_default()
        },
    );
    config
        .0
        .normalize_wire_vector_shapes()
        .map_err(SliceError::InvalidInput)?;
    if kind == ProfileKind::Machine {
        super::super::preset_normalization::normalize_machine(&mut config.0)?;
    }
    Ok((config, filament_id))
}

pub(super) fn identity(kind: ProfileKind) -> &'static str {
    match kind {
        ProfileKind::Process => "print_settings_id",
        ProfileKind::Machine => "printer_settings_id",
        ProfileKind::Filament => "filament_settings_id",
    }
}

fn version(value: &str) -> Result<String, SliceError> {
    crate::project::preset_version::parse(value)
        .ok_or_else(|| SliceError::InvalidInput(format!("invalid preset version: {value}")))
}
