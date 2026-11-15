use serde::{Deserialize, Deserializer, de::Error as _};

use crate::options::{ObjectOptionOverrides, RegionOptionOverrides};

mod object_metadata;
mod part_metadata;

fn is_part_structural_metadata(key: &str) -> bool {
    matches!(
        key,
        "name"
            | "volume_type"
            | "part_type"
            | "matrix"
            | "mesh_shared"
            | "source_file"
            | "source_object_id"
            | "source_volume_id"
            | "source_offset_x"
            | "source_offset_y"
            | "source_offset_z"
            | "source_in_inches"
            | "source_in_meters"
    )
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename = "config")]
pub(crate) struct ModelSettings {
    #[serde(rename = "object", default)]
    pub objects: Vec<ObjectSettings>,
    #[serde(rename = "plate", default)]
    pub plates: Vec<PlateSettings>,
    #[serde(rename = "assemble")]
    pub assemble: Option<AssembleSettings>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObjectSettings {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub overrides: ObjectOptionOverrides,
    pub region_overrides: RegionOptionOverrides,
    pub parts: Vec<PartSettings>,
}

#[derive(Deserialize)]
struct ObjectSettingsWire {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "$value", default)]
    children: Vec<ObjectChild>,
}

#[derive(Deserialize)]
struct ObjectMetadataWire {
    #[serde(rename = "@key")]
    key: Option<String>,
    #[serde(rename = "@value")]
    value: Option<String>,
}

#[derive(Deserialize)]
enum ObjectChild {
    #[serde(rename = "metadata")]
    Metadata(ObjectMetadataWire),
    #[serde(rename = "part")]
    Part(Box<PartSettings>),
    #[serde(other)]
    Other,
}

impl<'de> Deserialize<'de> for ObjectSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ObjectSettingsWire { id, children } = ObjectSettingsWire::deserialize(deserializer)?;
        let mut metadata = Vec::new();
        let mut parts = Vec::new();
        for child in children {
            match child {
                ObjectChild::Metadata(ObjectMetadataWire {
                    key: Some(key),
                    value,
                }) if !key.is_empty() => {
                    let value = value.ok_or_else(|| D::Error::missing_field("@value"))?;
                    metadata.push(Metadata { key, value });
                }
                ObjectChild::Part(part) => parts.push(*part),
                ObjectChild::Metadata(_) | ObjectChild::Other => {}
            }
        }
        Self::from_ordered_metadata(id, metadata, parts).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PartSettings {
    pub id: u32,
    pub subtype: String,
    pub region_overrides: RegionOptionOverrides,
    pub retained_metadata: Vec<Metadata>,
    pub mesh_stat: Option<MeshStatistics>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Metadata {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlateSettings {
    pub metadata: Vec<Metadata>,
    pub model_instances: Vec<PlateModelInstance>,
}

#[derive(Deserialize)]
struct PlateSettingsWire {
    #[serde(rename = "$value", default)]
    children: Vec<PlateChild>,
}

#[derive(Deserialize)]
enum PlateChild {
    #[serde(rename = "metadata")]
    Metadata(Metadata),
    #[serde(rename = "model_instance")]
    ModelInstance(PlateModelInstance),
    #[serde(other)]
    Other,
}

impl<'de> Deserialize<'de> for PlateSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PlateSettingsWire::deserialize(deserializer)?;
        let mut plate = Self {
            metadata: Vec::new(),
            model_instances: Vec::new(),
        };
        for child in wire.children {
            match child {
                PlateChild::Metadata(metadata) => plate.metadata.push(metadata),
                PlateChild::ModelInstance(instance) => plate.model_instances.push(instance),
                PlateChild::Other => {}
            }
        }
        Ok(plate)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct PlateModelInstance {
    #[serde(rename = "metadata", default)]
    pub metadata: Vec<Metadata>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AssembleSettings {
    #[serde(rename = "assemble_item", default)]
    pub items: Vec<AssembleItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
