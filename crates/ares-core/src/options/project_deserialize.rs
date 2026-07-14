use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::{
    ProjectSettings, project_settings::ProjectSettingsBuilder,
    typed_legacy::deserialize_project_field,
};

impl<'de> Deserialize<'de> for ProjectSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProjectSettingsVisitor)
    }
}

struct ProjectSettingsVisitor;

impl<'de> Visitor<'de> for ProjectSettingsVisitor {
    type Value = ProjectSettings;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca project settings")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProjectSettingsBuilder::default();

        while let Some(key) = map.next_key::<String>()? {
            if deserialize_project_field(&mut builder, &key, &mut map)? {
                continue;
            }
            if builder.deserialize_known_field(&key, &mut map)? {
                continue;
            }

            return Err(serde::de::Error::custom(format!(
                "unknown Orca project option {key}"
            )));
        }

        builder.apply_thumbnail_composite::<A::Error>()?;
        Ok(builder.resolve())
    }
}
