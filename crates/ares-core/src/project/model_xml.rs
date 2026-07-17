use serde::{Deserialize, Deserializer};

use crate::SliceError;

use super::transform::Transform3d;

pub(crate) const PRODUCTION_NAMESPACE: &str =
    "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
pub(crate) const MATERIAL_NAMESPACE: &str =
    "http://schemas.microsoft.com/3dmanufacturing/material/2015/02";

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "model")]
pub(crate) struct ModelDocument {
    #[serde(rename = "@unit", default)]
    pub unit: ModelUnit,
    #[serde(rename = "@requiredextensions", default)]
    pub required_extensions: String,
    #[serde(rename = "metadata", default)]
    pub metadata: Vec<ModelMetadata>,
    pub resources: Resources,
    pub build: Build,
}

impl ModelDocument {
    pub(crate) fn validate(&self) -> Result<(), SliceError> {
        for object in &self.resources.objects {
            match (&object.mesh, &object.components) {
                (Some(mesh), None) => mesh.validate(self.unit.millimeter_factor())?,
                (None, Some(_)) => {}
                _ => {
                    return Err(SliceError::InvalidInput(format!(
                        "project object {} must contain exactly one mesh or components element",
                        object.id
                    )));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ModelUnit {
    Micron,
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}

impl ModelUnit {
    pub(crate) const fn millimeter_factor(self) -> f32 {
        match self {
            Self::Micron => 0.001,
            Self::Millimeter => 1.0,
            Self::Centimeter => 10.0,
            Self::Inch => 25.4,
            Self::Foot => 304.8,
            Self::Meter => 1_000.0,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ModelMetadata {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text", default)]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Resources {
    #[serde(rename = "colorgroup", default)]
    pub color_groups: Vec<MaterialColorGroup>,
    #[serde(rename = "object", default)]
    pub objects: Vec<ModelObject>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct MaterialColorGroup {
    #[serde(rename = "@id")]
    pub id: i32,
    #[serde(rename = "color", default)]
    pub colors: Vec<MaterialColor>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct MaterialColor {
    #[serde(rename = "@color")]
    pub color: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ModelObject {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@pid", default, deserialize_with = "deserialize_pid")]
    pub pid: i32,
    #[serde(rename = "@type", default)]
    pub object_type: ModelObjectType,
    pub mesh: Option<Mesh>,
    pub components: Option<Components>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ModelObjectType {
    #[default]
    Model,
    Other,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Mesh {
    pub vertices: Vertices,
    pub triangles: Triangles,
}

impl Mesh {
    fn validate(&self, millimeter_factor: f32) -> Result<(), SliceError> {
        if self.vertices.vertices.is_empty() || self.triangles.triangles.is_empty() {
            return Ok(());
        }
        if self
            .vertices
            .vertices
            .iter()
            .flat_map(|vertex| [vertex.x, vertex.y, vertex.z])
            .any(|component| !component.is_finite() || !(component * millimeter_factor).is_finite())
        {
            return Err(SliceError::InvalidInput(
                "project mesh vertices must be finite".to_owned(),
            ));
        }
        let vertex_count = self.vertices.vertices.len();
        if self.triangles.triangles.iter().any(|triangle| {
            [triangle.v1, triangle.v2, triangle.v3]
                .into_iter()
                .any(|index| usize::try_from(index).map_or(true, |index| index >= vertex_count))
        }) {
            return Err(SliceError::InvalidInput(
                "project mesh triangle references a missing vertex".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Vertices {
    #[serde(rename = "vertex", default)]
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Vertex {
    #[serde(rename = "@x")]
    pub x: f32,
    #[serde(rename = "@y")]
    pub y: f32,
    #[serde(rename = "@z")]
    pub z: f32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Triangles {
    #[serde(rename = "triangle", default)]
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Triangle {
    #[serde(rename = "@v1")]
    pub v1: u32,
    #[serde(rename = "@v2")]
    pub v2: u32,
    #[serde(rename = "@v3")]
    pub v3: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Components {
    #[serde(rename = "component", default)]
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Component {
    #[serde(rename = "@path", default)]
    pub path: Option<String>,
    #[serde(rename = "@objectid")]
    pub object_id: u32,
    #[serde(
        rename = "@transform",
        default,
        deserialize_with = "deserialize_optional_transform"
    )]
    pub transform: Transform3d,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Build {
    #[serde(rename = "item", default)]
    pub items: Vec<BuildItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct BuildItem {
    #[serde(rename = "@path", default)]
    pub path: Option<String>,
    #[serde(rename = "@objectid")]
    pub object_id: u32,
    #[serde(
        rename = "@transform",
        default,
        deserialize_with = "deserialize_optional_transform"
    )]
    pub transform: Transform3d,
    #[serde(
        rename = "@printable",
        default = "default_true",
        deserialize_with = "deserialize_binary_bool"
    )]
    pub printable: bool,
    #[serde(
        rename = "@auto_drop",
        default = "default_true",
        deserialize_with = "deserialize_binary_bool"
    )]
    pub auto_drop: bool,
}

fn default_true() -> bool {
    true
}

fn deserialize_pid<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let unsigned = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(&value);
    let digit_count = unsigned
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    let sign_len = value.len() - unsigned.len();
    Ok(value[..sign_len + digit_count].parse().unwrap_or(0))
}

fn deserialize_optional_transform<'de, D>(deserializer: D) -> Result<Transform3d, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Ok(Transform3d::IDENTITY)
    } else {
        Transform3d::parse_3mf(&value).map_err(serde::de::Error::custom)
    }
}

fn deserialize_binary_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let value = value.trim_ascii_start().as_bytes();
    let digits = value
        .strip_prefix(b"+")
        .or_else(|| value.strip_prefix(b"-"))
        .unwrap_or(value);
    Ok(digits
        .iter()
        .take_while(|digit| digit.is_ascii_digit())
        .any(|digit| *digit != b'0'))
}
