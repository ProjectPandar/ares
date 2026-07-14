mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessIroningType, ProcessNoiseType, ProcessSeamScarfType,
    ProcessWallDirection, ProcessWallSequence,
};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, OrcaInts, OrcaString, OrcaStrings, Percent,
    option_group::declare_option_group,
    region_fields::{REGION_OPTION_DECLARATION_ORDER, region_option_fields},
};
use super::object_source::ProcessInfillPattern;

macro_rules! declare_process_region_source_options {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        declare_option_group! {
            pub struct ProcessRegionSourceOptions, ProcessRegionSourceOptionsBuilder {
                $($field => $key: $ty = $default),*
            }
        }
    };
}

region_option_fields!(declare_process_region_source_options);

impl ProcessRegionSourceOptionsBuilder {
    pub(crate) fn set_derived_is_infill_first(&mut self, value: OrcaBool) {
        self.is_infill_first = Some(value);
    }
}

impl ProcessRegionSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 149] = REGION_OPTION_DECLARATION_ORDER;
}

impl Default for ProcessRegionSourceOptions {
    fn default() -> Self {
        ProcessRegionSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessRegionSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RegionSourceVisitor)
    }
}

struct RegionSourceVisitor;

impl<'de> Visitor<'de> for RegionSourceVisitor {
    type Value = ProcessRegionSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca process region-source options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessRegionSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca process region option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

fn string(value: &str) -> OrcaString {
    OrcaString(value.to_owned())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}
