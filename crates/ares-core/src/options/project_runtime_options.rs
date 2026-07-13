mod gcode_source;
mod preset_source;
mod print_source;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub(crate) use gcode_source::ProjectGCodeSourceOptionsBuilder;
pub use gcode_source::{ProjectFilamentMapMode, ProjectGCodeSourceOptions};
pub use preset_source::ProjectPresetSourceOptions;
pub(crate) use preset_source::ProjectPresetSourceOptionsBuilder;
pub(crate) use print_source::ProjectPrintSourceOptionsBuilder;
pub use print_source::{ProjectBedType, ProjectPrintSourceOptions};

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectRuntimeOptions {
    pub gcode: ProjectGCodeSourceOptions,
    pub print: ProjectPrintSourceOptions,
    pub preset: ProjectPresetSourceOptions,
}

impl Default for ProjectRuntimeOptions {
    fn default() -> Self {
        ProjectRuntimeOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProjectRuntimeOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProjectRuntimeVisitor)
    }
}

struct ProjectRuntimeVisitor;

impl<'de> Visitor<'de> for ProjectRuntimeVisitor {
    type Value = ProjectRuntimeOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca project runtime options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProjectRuntimeOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca project runtime option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct ProjectRuntimeOptionsBuilder {
    gcode: ProjectGCodeSourceOptionsBuilder,
    print: ProjectPrintSourceOptionsBuilder,
    preset: ProjectPresetSourceOptionsBuilder,
}

impl ProjectRuntimeOptionsBuilder {
    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.gcode.deserialize_known_field(key, map)?
            || self.print.deserialize_known_field(key, map)?
        {
            Ok(true)
        } else {
            self.preset.deserialize_known_field(key, map)
        }
    }

    pub(crate) fn resolve(self) -> ProjectRuntimeOptions {
        ProjectRuntimeOptions {
            gcode: self.gcode.resolve(),
            print: self.print.resolve(),
            preset: self.preset.resolve(),
        }
    }
}
