mod object_source;
mod region_source;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub(crate) use object_source::ProcessObjectSourceOptionsBuilder;
pub use object_source::{
    ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessObjectSourceOptions, ProcessPerimeterGenerator,
    ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
};
pub(crate) use region_source::ProcessRegionSourceOptionsBuilder;
pub use region_source::{
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessIroningType, ProcessNoiseType, ProcessRegionSourceOptions,
    ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessOptions {
    pub object: ProcessObjectSourceOptions,
    pub region: ProcessRegionSourceOptions,
}

impl<'de> Deserialize<'de> for ProcessOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProcessVisitor)
    }
}

struct ProcessVisitor;

impl<'de> Visitor<'de> for ProcessVisitor {
    type Value = ProcessOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca process options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.object.deserialize_known_field(&key, &mut map)?
                && !builder.region.deserialize_known_field(&key, &mut map)?
            {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca process option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
struct ProcessOptionsBuilder {
    object: ProcessObjectSourceOptionsBuilder,
    region: ProcessRegionSourceOptionsBuilder,
}

impl ProcessOptionsBuilder {
    fn resolve(self) -> ProcessOptions {
        ProcessOptions {
            object: self.object.resolve(),
            region: self.region.resolve(),
        }
    }
}
