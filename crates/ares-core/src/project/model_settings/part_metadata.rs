use serde::{Deserialize, Deserializer, de::Error as _};

use crate::{SliceError, options::RegionOptionOverrides};

use super::{MeshStatistics, Metadata, PartSettings};

#[derive(Deserialize)]
struct PartSettingsWire {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "@subtype")]
    subtype: String,
    #[serde(rename = "metadata", default)]
    metadata: Vec<Metadata>,
    #[serde(rename = "mesh_stat")]
    mesh_stat: Option<MeshStatistics>,
}

impl<'de> Deserialize<'de> for PartSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let PartSettingsWire {
            id,
            subtype,
            metadata,
            mesh_stat,
        } = PartSettingsWire::deserialize(deserializer)?;
        Self::from_ordered_metadata(id, subtype, metadata, mesh_stat).map_err(D::Error::custom)
    }
}

impl PartSettings {
    fn from_ordered_metadata(
        id: u32,
        subtype: String,
        metadata: Vec<Metadata>,
        mesh_stat: Option<MeshStatistics>,
    ) -> Result<Self, SliceError> {
        let mut region_overrides = RegionOptionOverrides::default();
        let mut retained_metadata = Vec::with_capacity(metadata.len());
        for entry in metadata {
            if !region_overrides.deserialize_known_field(&entry.key, &entry.value)? {
                retained_metadata.push(entry);
            }
        }
        Ok(Self {
            id,
            subtype,
            region_overrides,
            retained_metadata,
            mesh_stat,
        })
    }
}
