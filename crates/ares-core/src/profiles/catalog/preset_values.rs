//! Resolved preset option values (`Preset::config` after loading).

use std::collections::BTreeMap;

use super::{Catalog, invalid};
use crate::SliceError;
use crate::profiles::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

impl Catalog {
    /// `keys` of the preset at `position` as upstream holds them after load:
    /// the parent's config with the preset's own values applied
    /// (`PresetBundle.cpp:4885-4927`, `config = *default_config;
    /// config.apply(config_src)`), down the whole `inherits` chain. A root
    /// preset starts from the default preset (`:4919-4924`), so a key no
    /// chain member sets has its `PrintConfigDef` default. The chain is
    /// resolved here rather than stored, so the
    /// `extend_default_config_length` vector padding (`:4928`) is not
    /// applied.
    pub(in crate::profiles) fn resolved_values(
        &self,
        position: usize,
        keys: &[&str],
    ) -> Result<BTreeMap<String, RawConfigValue>, SliceError> {
        let chain = self.chain_positions(position)?;
        keys.iter()
            .map(|key| {
                let definition = crate::options::registry::option_definition(key)
                    .ok_or_else(|| invalid(format!("unknown option '{key}'")))?;
                let value = chain
                    .iter()
                    .rev()
                    .find_map(|member| self.document(*member).fields.get(*key))
                    .cloned()
                    .unwrap_or_else(|| RawConfigValue::Text(definition.default_value.to_owned()));
                Ok(((*key).to_owned(), value))
            })
            .collect()
    }
}

/// `PresetCatalogHandle::preset_values` result: one resolved key map per
/// requested preset, serialized as a JSON array of objects whose values keep
/// the preset document shape (string or string array).
#[derive(Debug, serde::Serialize)]
#[serde(transparent)]
pub struct PresetValues(Vec<BTreeMap<String, RawConfigValue>>);

impl super::PresetCatalogHandle {
    /// The resolved `keys` of each named selectable preset of `kind`, in
    /// `names` order. A value is the stored document value of the nearest
    /// chain member that sets the key, else the option's default text.
    /// Rejects a name that is not selectable or misses a parent (upstream
    /// never loads such a preset, `Preset.cpp:1685-1691`) and an unknown key.
    pub fn preset_values(
        &self,
        kind: ProfileKind,
        names: &[String],
        keys: &[&str],
    ) -> Result<PresetValues, SliceError> {
        names
            .iter()
            .map(|name| {
                let position = self
                    .catalog
                    .selectable_position(kind, name)
                    .filter(|position| self.catalog.missing_parent(*position).is_none())
                    .ok_or_else(|| invalid(format!("preset '{name}' is not in the catalog")))?;
                self.catalog.resolved_values(position, keys)
            })
            .collect::<Result<_, _>>()
            .map(PresetValues)
    }
}
