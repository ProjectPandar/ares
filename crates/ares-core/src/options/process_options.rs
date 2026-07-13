mod object_source;

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

pub(crate) use object_source::ProcessObjectSourceOptionsBuilder;
pub use object_source::{
    ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessObjectSourceOptions, ProcessPerimeterGenerator,
    ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessOptions {
    pub object: ProcessObjectSourceOptions,
}

impl<'de> Deserialize<'de> for ProcessOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProcessVisitor)
    }
}

impl Serialize for ProcessOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.object.serialize(serializer)
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
            if !builder.object.deserialize_known_field(&key, &mut map)? {
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
}

impl ProcessOptionsBuilder {
    fn resolve(self) -> ProcessOptions {
        ProcessOptions {
            object: self.object.resolve(),
        }
    }
}
