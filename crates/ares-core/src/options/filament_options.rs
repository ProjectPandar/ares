mod gcode_source;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use gcode_source::FilamentGCodeSourceOptions;
use gcode_source::FilamentGCodeSourceOptionsBuilder;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilamentOptions {
    pub gcode: FilamentGCodeSourceOptions,
}

impl<'de> Deserialize<'de> for FilamentOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FilamentVisitor)
    }
}

struct FilamentVisitor;

impl<'de> Visitor<'de> for FilamentVisitor {
    type Value = FilamentOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut gcode = FilamentGCodeSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !gcode.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &FilamentGCodeSourceOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(FilamentOptions {
            gcode: gcode.resolve(),
        })
    }
}
