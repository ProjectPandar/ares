use serde::{Deserialize, Deserializer, de::Error as _};

use crate::Transform3d;
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
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
    pub auto_drop: Option<String>,
    pub name: String,
    pub module: String,
    pub overrides: ObjectOptionOverrides,
    pub region_overrides: RegionOptionOverrides,
    pub options: Vec<Metadata>,
    pub parts: Vec<PartSettings>,
}

#[derive(Deserialize)]
struct ObjectSettingsWire {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "@auto_drop")]
    auto_drop: Option<String>,
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
        let ObjectSettingsWire {
            id,
            auto_drop,
            children,
        } = ObjectSettingsWire::deserialize(deserializer)?;
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
        Self::from_ordered_metadata(id, auto_drop, metadata, parts).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PartSettings {
    pub id: u32,
    pub subtype: String,
    pub region_overrides: RegionOptionOverrides,
    pub retained_metadata: Vec<Metadata>,
    pub options: Vec<Metadata>,
    pub mesh_stat: Option<MeshStatistics>,
    pub has_parametric_shape: bool,
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
    #[serde(rename = "@object_id", default, deserialize_with = "assemble_id")]
    pub object_id: i32,
    #[serde(rename = "@instance_id", default, deserialize_with = "assemble_id")]
    pub instance_id: i32,
    #[serde(rename = "@transform", default)]
    transform: String,
    #[serde(rename = "@offset", default)]
    offset: String,
}

impl AssembleItem {
    pub(super) fn state(&self) -> (Transform3d, [f64; 3]) {
        let transform = assemble_values(&self.transform)
            .map_or(Transform3d::IDENTITY, Transform3d::from_3mf_values);
        (transform, assemble_values(&self.offset).unwrap_or_default())
    }
}

// bbs_3mf.cpp:552–558 uses an optionally signed integer prefix with no whitespace skipper.
fn assemble_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    let text = String::deserialize(deserializer)?;
    let sign = number_sign_len(&text);
    let end = sign + decimal_digits(&text[sign..]);
    Ok(text[..end].parse().unwrap_or_default())
}

fn assemble_values<const N: usize>(text: &str) -> Option<[f64; N]> {
    // bbs_3mf.cpp:637–682 compresses runs of spaces but keeps empty edge tokens.
    let mut tokens = text
        .starts_with(' ')
        .then_some("")
        .into_iter()
        .chain(text.split(' ').filter(|token| !token.is_empty()))
        .chain(text.ends_with(' ').then_some(""));
    let mut values = [0.0; N];
    for value in &mut values {
        *value = assemble_number(tokens.next()?);
    }
    tokens.next().is_none().then_some(values)
}

fn assemble_number(token: &str) -> f64 {
    let token = token.trim_ascii_start();
    if let Ok(value) = token.parse() {
        return value;
    }
    let sign = number_sign_len(token);
    if let Some(prefix) = token.get(sign..sign + 3)
        && (prefix.eq_ignore_ascii_case("inf") || prefix.eq_ignore_ascii_case("nan"))
    {
        return token[..sign + 3].parse().unwrap_or_default();
    }
    // atof accepts a decimal prefix and leaves an incomplete exponent unread.
    let mut end = sign + decimal_digits(&token[sign..]);
    if let Some(fraction) = token[end..].strip_prefix('.') {
        end += 1 + decimal_digits(fraction);
    }
    if let Some(exponent) = token[end..].strip_prefix(['e', 'E']) {
        let exponent_sign = number_sign_len(exponent);
        let digits = decimal_digits(&exponent[exponent_sign..]);
        if digits != 0 {
            end += 1 + exponent_sign + digits;
        }
    }
    token[..end].parse().unwrap_or_default()
}

fn number_sign_len(token: &str) -> usize {
    usize::from(token.starts_with(['-', '+']))
}

fn decimal_digits(token: &str) -> usize {
    token.bytes().take_while(u8::is_ascii_digit).count()
}
