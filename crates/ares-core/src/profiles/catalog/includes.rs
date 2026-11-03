//! PresetBundle::resolve_vendor_preset: parent, ordered include diffs, then own keys.

mod defaults;
mod equality;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use super::{Catalog, ProfileDocument, Scope, invalid};
use crate::{
    SliceError,
    options::{json_deserialize::deserialize_preset_config, registry::option_definition},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

impl Catalog {
    pub(in crate::profiles) fn resolved_layers(
        &self,
        position: usize,
    ) -> Result<Vec<Cow<'_, ProfileDocument>>, SliceError> {
        let mut layers = Vec::new();
        self.collect_layers(position, &mut BTreeSet::new(), &mut layers)?;
        Ok(layers)
    }

    fn collect_layers<'a>(
        &'a self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
        layers: &mut Vec<Cow<'a, ProfileDocument>>,
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
            self.collect_layers(parent_position, visiting, layers)?;
        }
        layers.push(self.resolved_document(position, visiting)?);
        visiting.remove(&position);
        Ok(())
    }

    fn resolved_document<'a>(
        &'a self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
    ) -> Result<Cow<'a, ProfileDocument>, SliceError> {
        let record = &self.records[position];
        if !matches!(record.scope, Scope::Vendor(_)) || record.document.includes.is_empty() {
            return Ok(Cow::Borrowed(&record.document));
        }
        let mut document = record.document.clone();
        let mut fields = BTreeMap::new();
        for name in &record.document.includes {
            // PresetBundle.cpp:7632: only earlier non-instantiable registrations
            // in this vendor/type load are available in include_maps.
            let included = self.include_position(position, name)?;
            fields.extend(self.include_diff(included, visiting)?);
        }
        // entry.config_src has already passed handle_legacy before Native overlays it.
        let mut own = ProjectSettingsRaw::default();
        own.extend(document.fields);
        own.handle_legacy();
        fields.extend(
            own.entries()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        document.fields = fields;
        Ok(Cow::Owned(document))
    }

    fn include_position(&self, position: usize, name: &str) -> Result<usize, SliceError> {
        let record = &self.records[position];
        let owner = &record.document.name;
        let Some(&included) = self
            .index
            .get(&(record.scope, record.kind, name.to_owned()))
        else {
            return Err(invalid(format!(
                "include profile '{name}' was not found for '{owner}'"
            )));
        };
        if included >= position
            || self.records[included].document.instantiation.as_deref() != Some("false")
        {
            return Err(invalid(format!(
                "include profile '{name}' was not available before '{owner}'"
            )));
        }
        Ok(included)
    }

    // PresetBundle.cpp:7645-7648: snapshot the resolved config's diff against
    // its collection default before this template's variant padding.
    fn include_diff(
        &self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
    ) -> Result<BTreeMap<String, RawConfigValue>, SliceError> {
        let mut layers = Vec::new();
        self.collect_layers(position, visiting, &mut layers)?;
        let mut raw = ProjectSettingsRaw::default();
        for layer in layers {
            raw.extend(
                layer
                    .fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        raw.handle_legacy();
        let kind = self.records[position].kind;
        let mut fields = raw
            .entries()
            .filter(|(key, _)| defaults::contains(kind, key) && option_definition(key).is_some())
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        let baseline = fields
            .keys()
            .map(|key| {
                let definition = option_definition(key).unwrap();
                (
                    key.clone(),
                    RawConfigValue::Text(definition.default_wire_value.to_owned()),
                )
            })
            .collect();
        let decoded: BTreeMap<String, RawConfigValue> =
            deserialize_preset_config(RawConfigValue::Object(fields.clone())).map_err(invalid)?;
        let baseline: BTreeMap<String, RawConfigValue> =
            deserialize_preset_config(RawConfigValue::Object(baseline)).map_err(invalid)?;
        for (key, value) in &decoded {
            let kind = option_definition(key).unwrap().kind;
            if equality::equal(kind, value, &baseline[key]).map_err(invalid)? {
                fields.remove(key);
            }
        }
        Ok(fields)
    }
}
