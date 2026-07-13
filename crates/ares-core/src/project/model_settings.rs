use serde::{Deserialize, Deserializer, de::Error as _};

use crate::options::ObjectOptionOverrides;

mod object_metadata;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "config")]
pub(crate) struct ModelSettings {
    #[serde(rename = "object", default)]
    pub objects: Vec<ObjectSettings>,
    #[serde(rename = "plate", default)]
    pub plates: Vec<PlateSettings>,
    #[serde(rename = "assemble")]
    pub assemble: Option<AssembleSettings>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ObjectSettings {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub overrides: ObjectOptionOverrides,
    pub retained_config: Vec<Metadata>,
    pub parts: Vec<PartSettings>,
}

#[derive(Deserialize)]
struct ObjectSettingsWire {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "metadata", default)]
    metadata: Vec<Metadata>,
    #[serde(rename = "part", default)]
    parts: Vec<PartSettings>,
}

impl<'de> Deserialize<'de> for ObjectSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ObjectSettingsWire {
            id,
            metadata,
            parts,
        } = ObjectSettingsWire::deserialize(deserializer)?;
        Self::from_ordered_metadata(id, metadata, parts).map_err(D::Error::custom)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PartSettings {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@subtype")]
    pub subtype: String,
    #[serde(rename = "metadata", default)]
    pub metadata: Vec<Metadata>,
    #[serde(rename = "mesh_stat")]
    pub mesh_stat: Option<MeshStatistics>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Metadata {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct MeshStatistics {
    #[serde(rename = "@edges_fixed")]
    pub edges_fixed: u32,
    #[serde(rename = "@degenerate_facets")]
    pub degenerate_facets: u32,
    #[serde(rename = "@facets_removed")]
    pub facets_removed: u32,
    #[serde(rename = "@facets_reversed")]
    pub facets_reversed: u32,
    #[serde(rename = "@backwards_edges")]
    pub backwards_edges: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PlateSettings {
    #[serde(rename = "metadata", default)]
    pub metadata: Vec<Metadata>,
    #[serde(rename = "model_instance", default)]
    pub model_instances: Vec<PlateModelInstance>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PlateModelInstance {
    #[serde(rename = "metadata", default)]
    pub metadata: Vec<Metadata>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct AssembleSettings {
    #[serde(rename = "assemble_item", default)]
    pub items: Vec<AssembleItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct AssembleItem {
    #[serde(rename = "@object_id")]
    pub object_id: u32,
    #[serde(rename = "@instance_id")]
    pub instance_id: u32,
    #[serde(rename = "@transform")]
    pub transform: String,
    #[serde(rename = "@offset")]
    pub offset: String,
}
