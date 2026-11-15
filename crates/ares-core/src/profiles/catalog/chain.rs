//! Chain inspection: selectable lookup, scope/inherits metadata, and
//! the parent-first chain walk (`PresetBundle` inherits semantics).

use std::collections::BTreeSet;

use super::Catalog;
use super::Scope;
use super::{ProfileDocument, invalid};
use crate::SliceError;
use crate::profiles::ProfileKind;

pub(in crate::profiles) struct MissingParentLink {
    pub(in crate::profiles) kind: ProfileKind,
    pub(in crate::profiles) name: String,
    pub(in crate::profiles) parent: String,
}

impl Catalog {
    pub(in crate::profiles) fn selectable_position(
        &self,
        kind: ProfileKind,
        name: &str,
    ) -> Option<usize> {
        // Preset.cpp:3894-3909: ordinary suffix lookup precedes the default prefix.
        self.selectable
            .get(&(kind, name.to_owned()))
            .copied()
            .or_else(|| {
                let position = self.default_position(kind);
                (self.document(position).name == name).then_some(position)
            })
    }

    pub(in crate::profiles) fn selectable_positions(&self, kind: ProfileKind) -> Vec<usize> {
        // Upstream iterates `m_presets` in load order (`Preset.cpp:3355`); the
        // record vector preserves vendor-then-index insertion order.
        self.records
            .iter()
            .enumerate()
            .filter(|(position, record)| {
                record.kind == kind
                    && self.selectable.get(&(kind, record.document.name.clone())) == Some(position)
            })
            .map(|(position, _)| position)
            .collect()
    }

    pub(in crate::profiles) fn is_user_scope(&self, position: usize) -> bool {
        self.records[position].scope == Scope::User
    }

    pub(in crate::profiles) fn is_default(&self, position: usize) -> bool {
        self.records[position].scope == Scope::Default
    }

    /// The record's own `inherits` value (`Preset::inherits` reads the stored
    /// option; `Preset.hpp:320-321`).
    pub(in crate::profiles) fn leaf_inherits(&self, position: usize) -> Option<String> {
        self.records[position]
            .document
            .inherits
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    pub(in crate::profiles) fn vendor_name(&self, position: usize) -> &str {
        match self.records[position].scope {
            Scope::Vendor(index) => &self.vendor_names[index],
            Scope::Default | Scope::User => "",
        }
    }

    pub(in crate::profiles) fn document(&self, position: usize) -> &ProfileDocument {
        &self.records[position].document
    }

    /// The first unresolvable `inherits` link on the preset's chain, reported
    /// as (holder kind, holder name, missing parent). Upstream skips such
    /// presets at load (`Preset.cpp:1685-1691`, `PresetBundle.cpp:1521-1525`);
    /// the handle keeps them but they stay invisible to filtering until the
    /// parent arrives.
    pub(in crate::profiles) fn missing_parent(&self, position: usize) -> Option<MissingParentLink> {
        let mut position = position;
        let mut steps = self.records.len() + 1;
        loop {
            // Cycle guard: `collect_positions` reports inheritance cycles as
            // errors; this walk only needs the first missing link.
            steps = steps.saturating_sub(1);
            if steps == 0 {
                return None;
            }
            let record = &self.records[position];
            let parent = record
                .document
                .inherits
                .as_deref()
                .filter(|name| !name.is_empty())?;
            match self.parent_position(record, parent) {
                Some(next) => position = next,
                None => {
                    return Some(MissingParentLink {
                        kind: record.kind,
                        name: record.document.name.clone(),
                        parent: parent.to_owned(),
                    });
                }
            }
        }
    }

    /// Parent-first record positions for the preset at `position`.
    pub(in crate::profiles) fn chain_positions(
        &self,
        position: usize,
    ) -> Result<Vec<usize>, SliceError> {
        let mut chain = Vec::new();
        self.collect_positions(position, &mut BTreeSet::new(), &mut chain)?;
        Ok(chain)
    }

    pub(in crate::profiles) fn collect_positions(
        &self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
        chain: &mut Vec<usize>,
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
            self.collect_positions(parent_position, visiting, chain)?;
        }
        chain.push(position);
        Ok(())
    }
}
