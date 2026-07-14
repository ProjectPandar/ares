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
        let mut builder = PresetMetadataBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(&key, &FIELDS));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct PresetMetadataBuilder {
    from: Option<String>,
    name: Option<String>,
    version: Option<String>,
}

impl PresetMetadataBuilder {
    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: MapAccess<'de>,
    {
        let field = match key {
            "from" => &mut self.from,
            "name" => &mut self.name,
            "version" => &mut self.version,
            _ => return Ok(false),
        };
        if field.is_some() {
            return Err(serde::de::Error::custom(format!(
                "duplicate Orca option {key}"
            )));
        }
        *field = Some(map.next_value::<String>().map_err(|error| {
            serde::de::Error::custom(format_args!("invalid Orca preset metadata {key}: {error}"))
        })?);
        Ok(true)
    }

    pub(crate) fn resolve(self) -> PresetMetadata {
        PresetMetadata {
            from: self.from.unwrap_or_default(),
            name: self.name.unwrap_or_default(),
            version: self.version.unwrap_or_default(),
        }
    }
}
