use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

use super::{Metadata, ObjectSettings, PartSettings};

impl ObjectSettings {
    pub(super) fn from_ordered_metadata(
        id: u32,
        metadata: Vec<Metadata>,
        parts: Vec<PartSettings>,
    ) -> Result<Self, SliceError> {
        let mut name = String::new();
        let mut module = String::new();
        let mut overrides = ObjectOptionOverrides::default();
        let mut region_overrides = RegionOptionOverrides::default();
        let mut retained_config = Vec::with_capacity(metadata.len());
        for entry in metadata {
            match entry.key.as_str() {
                "name" => name = entry.value,
                "module" => module = entry.value,
                key if overrides.deserialize_known_field(key, &entry.value)? => {}
                key if region_overrides.deserialize_known_field(key, &entry.value)? => {}
                _ => retained_config.push(entry),
            }
        }
        Ok(Self {
            id,
            name,
            module,
            overrides,
            region_overrides,
            retained_config,
            parts,
        })
    }
}
