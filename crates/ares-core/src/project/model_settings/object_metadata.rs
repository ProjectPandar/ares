use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides, deserialize_object_model_field},
};

use super::{Metadata, ObjectSettings, PartSettings};

impl ObjectSettings {
    pub(super) fn from_ordered_metadata(
        id: u32,
        auto_drop: Option<String>,
        metadata: Vec<Metadata>,
        parts: Vec<PartSettings>,
    ) -> Result<Self, SliceError> {
        let mut name = String::new();
        let mut module = String::new();
        let mut overrides = ObjectOptionOverrides::default();
        let mut region_overrides = RegionOptionOverrides::default();
        let mut options = Vec::new();
        for entry in metadata {
            match entry.key.as_str() {
                "name" => {
                    name = entry.value;
                    continue;
                }
                "module" => {
                    module = entry.value;
                    continue;
                }
                _ => {}
            }
            if let Some((key, value)) = deserialize_object_model_field(
                entry.key,
                entry.value,
                &mut overrides,
                &mut region_overrides,
            )? {
                options.push(Metadata { key, value });
            }
        }
        Ok(Self {
            id,
            auto_drop,
            name,
            module,
            overrides,
            region_overrides,
            options,
            parts,
        })
    }
}
