pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::super::{Nullable, OrcaFloat, Percent, option_group::declare_option_group};

declare_option_group! {
    append pub struct FilamentRegionSourceOptions, FilamentRegionSourceOptionsBuilder {
        filament_ironing_flow => "filament_ironing_flow": Vec<Nullable<Percent>> = nil_vector(),
        filament_ironing_spacing => "filament_ironing_spacing": Vec<Nullable<OrcaFloat>> = nil_vector(),
        filament_ironing_inset => "filament_ironing_inset": Vec<Nullable<OrcaFloat>> = nil_vector(),
        filament_ironing_speed => "filament_ironing_speed": Vec<Nullable<OrcaFloat>> = nil_vector(),
    }
}

impl FilamentRegionSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 4] = [
        "filament_ironing_flow",
        "filament_ironing_spacing",
        "filament_ironing_inset",
        "filament_ironing_speed",
    ];
}

impl Default for FilamentRegionSourceOptions {
    fn default() -> Self {
        FilamentRegionSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentRegionSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RegionSourceVisitor)
    }
}

struct RegionSourceVisitor;

impl<'de> Visitor<'de> for RegionSourceVisitor {
    type Value = FilamentRegionSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca PrintRegionConfig filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentRegionSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &Self::Value::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn nil_vector<T>() -> Vec<Nullable<T>> {
    vec![Nullable::Nil]
}
