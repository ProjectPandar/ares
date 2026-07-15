use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides, deserialize_object_model_field},
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
        for entry in metadata {
            match entry.key.as_str() {
                "name" => name = entry.value,
                "module" => module = entry.value,
                _ => {
                    deserialize_object_model_field(
                        entry.key,
                        entry.value,
                        &mut overrides,
                        &mut region_overrides,
                    )?;
                }
            }
        }
        Ok(Self {
            id,
            name,
            module,
            overrides,
            region_overrides,
            parts,
        })
    }
}
