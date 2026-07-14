mod gcode_source;
mod print_source;
mod region_source;
mod retract_overrides;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use gcode_source::FilamentGCodeSourceOptions;
use gcode_source::FilamentGCodeSourceOptionsBuilder;
use print_source::FilamentPrintSourceOptionsBuilder;
pub use print_source::{FilamentPrintSourceOptions, RawOverhangFanThreshold};
pub use region_source::FilamentRegionSourceOptions;
use region_source::FilamentRegionSourceOptionsBuilder;
pub use retract_overrides::FilamentRetractOverrideOptions;
use retract_overrides::FilamentRetractOverrideOptionsBuilder;

use super::{OrcaFloat, OrcaFloats};

#[derive(Clone, Debug, PartialEq)]
pub struct FilamentOptions {
    pub gcode: FilamentGCodeSourceOptions,
    pub print: FilamentPrintSourceOptions,
    pub region: FilamentRegionSourceOptions,
    pub retract_overrides: FilamentRetractOverrideOptions,
    pub pellet_flow_coefficient: OrcaFloats,
}

impl Default for FilamentOptions {
    fn default() -> Self {
        FilamentOptionsBuilder::default().resolve()
    }
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
        let mut builder = FilamentOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca filament option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct FilamentOptionsBuilder {
    gcode: FilamentGCodeSourceOptionsBuilder,
    print: FilamentPrintSourceOptionsBuilder,
    region: FilamentRegionSourceOptionsBuilder,
    retract_overrides: FilamentRetractOverrideOptionsBuilder,
    pellet_flow_coefficient: Option<OrcaFloats>,
}

impl FilamentOptionsBuilder {
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
            || self.region.deserialize_known_field(key, map)?
            || self.retract_overrides.deserialize_known_field(key, map)?
        {
            Ok(true)
        } else {
            self.deserialize_direct_field(key, map)
        }
    }

    fn deserialize_direct_field<'de, A>(&mut self, key: &str, map: &mut A) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if key != "pellet_flow_coefficient" {
            return Ok(false);
        }
        if self.pellet_flow_coefficient.is_some() {
            return Err(serde::de::Error::custom(
                "duplicate Orca option pellet_flow_coefficient",
            ));
        }
        self.pellet_flow_coefficient = Some(map.next_value::<OrcaFloats>().map_err(|error| {
            serde::de::Error::custom(format_args!(
                "invalid Orca option pellet_flow_coefficient: {error}"
            ))
        })?);
        Ok(true)
    }

    pub(crate) fn resolve(self) -> FilamentOptions {
        FilamentOptions {
            gcode: self.gcode.resolve(),
            print: self.print.resolve(),
            region: self.region.resolve(),
            retract_overrides: self.retract_overrides.resolve(),
            pellet_flow_coefficient: self
                .pellet_flow_coefficient
                .unwrap_or_else(|| OrcaFloats(vec![OrcaFloat(0.4157)])),
        }
    }
}
