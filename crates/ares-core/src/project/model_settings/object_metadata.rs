use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides, deserialize_object_model_field},
};

use super::{Metadata, ObjectSettings, PartSettings, is_structural_metadata};

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
                key if is_structural_metadata(key) => retained_config.push(entry),
                _ => {
                    let retained = deserialize_object_model_field(
                        entry.key,
                        entry.value,
                        &mut overrides,
                        &mut region_overrides,
                    )?
                    .map(|(key, value)| Metadata { key, value });
                    retained_config.extend(retained);
                }
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
