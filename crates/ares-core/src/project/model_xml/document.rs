//! bbs_3mf.cpp:3582-3593,4142-4160: metadata may surround structural children.

use serde::{
    Deserialize, Deserializer,
    de::{Error as _, IgnoredAny},
};

use super::{Build, ModelDocument, ModelMetadata, ModelUnit, Resources};

#[derive(Deserialize)]
#[serde(bound(deserialize = "B: Deserialize<'de>"))]
struct DocumentWire<B> {
    #[serde(rename = "@unit", default)]
    unit: ModelUnit,
    #[serde(rename = "@requiredextensions", default)]
    required_extensions: String,
    #[serde(rename = "$value", default)]
    children: Vec<DocumentChild<B>>,
}

#[derive(Deserialize)]
enum DocumentChild<B> {
    #[serde(rename = "metadata")]
    Metadata(ModelMetadata),
    #[serde(rename = "resources")]
    Resources(Resources),
    #[serde(rename = "build")]
    Build(B),
    #[serde(other)]
    Other,
}

fn read_document<'de, D, B>(deserializer: D) -> Result<ModelDocument<Option<B>>, D::Error>
where
    D: Deserializer<'de>,
    B: Deserialize<'de>,
{
    let wire = DocumentWire::<B>::deserialize(deserializer)?;
    let mut metadata = Vec::new();
    let mut resources = None;
    let mut build = None;
    for child in wire.children {
        match child {
            DocumentChild::Metadata(entry) => metadata.push(entry),
            DocumentChild::Resources(value) => {
                if resources.replace(value).is_some() {
                    return Err(D::Error::duplicate_field("resources"));
                }
            }
            DocumentChild::Build(value) => {
                if build.replace(value).is_some() {
                    return Err(D::Error::duplicate_field("build"));
                }
            }
            DocumentChild::Other => {}
        }
    }
    Ok(ModelDocument {
        unit: wire.unit,
        required_extensions: wire.required_extensions,
        metadata,
        resources: resources.ok_or_else(|| D::Error::missing_field("resources"))?,
        build,
    })
}

impl<'de> Deserialize<'de> for ModelDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ModelDocument {
            unit,
            required_extensions,
            metadata,
            resources,
            build,
        } = read_document::<D, Build>(deserializer)?;
        Ok(Self {
            unit,
            required_extensions,
            metadata,
            resources,
            build: build.ok_or_else(|| D::Error::missing_field("build"))?,
        })
    }
}

impl<'de> Deserialize<'de> for ModelDocument<Option<IgnoredAny>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // ObjectImporter ignores resource-document build/item payloads.
        read_document::<D, IgnoredAny>(deserializer)
    }
}
