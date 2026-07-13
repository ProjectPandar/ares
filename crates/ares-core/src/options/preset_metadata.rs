use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};

const FIELDS: [&str; 3] = ["from", "name", "version"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PresetMetadata {
    pub from: String,
    pub name: String,
    pub version: String,
}

impl Serialize for PresetMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(FIELDS.len()))?;
        map.serialize_entry("from", &self.from)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("version", &self.version)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for PresetMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PresetMetadataVisitor)
    }
}

struct PresetMetadataVisitor;

impl<'de> Visitor<'de> for PresetMetadataVisitor {
    type Value = PresetMetadata;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca preset metadata")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut from = None;
        let mut name = None;
        let mut version = None;

        while let Some(key) = map.next_key::<String>()? {
            let value = match key.as_str() {
                "from" if from.is_some() => {
                    return Err(serde::de::Error::duplicate_field("from"));
                }
                "name" if name.is_some() => {
                    return Err(serde::de::Error::duplicate_field("name"));
                }
                "version" if version.is_some() => {
                    return Err(serde::de::Error::duplicate_field("version"));
                }
                "from" | "name" | "version" => map.next_value::<String>().map_err(|error| {
                    serde::de::Error::custom(format_args!(
                        "invalid Orca preset metadata {key}: {error}"
                    ))
                })?,
                _ => return Err(serde::de::Error::unknown_field(&key, &FIELDS)),
            };

            match key.as_str() {
                "from" => from = Some(value),
                "name" => name = Some(value),
                "version" => version = Some(value),
                _ => unreachable!(),
            }
        }

        Ok(PresetMetadata {
            from: from.unwrap_or_default(),
            name: name.unwrap_or_default(),
            version: version.unwrap_or_default(),
        })
    }
}
