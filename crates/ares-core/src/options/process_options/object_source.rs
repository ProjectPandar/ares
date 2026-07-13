mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{
    ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessPerimeterGenerator, ProcessSeamPosition,
    ProcessSlicingMode, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle, ProcessSupportType,
};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent,
    object_fields::{OBJECT_OPTION_DECLARATION_ORDER, object_option_fields},
    option_group::declare_option_group,
};

macro_rules! declare_process_object_source_options {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        declare_option_group! {
            pub struct ProcessObjectSourceOptions, ProcessObjectSourceOptionsBuilder {
                $($field => $key: $ty = $default),*
            }
        }
    };
}

object_option_fields!(declare_process_object_source_options);

impl ProcessObjectSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 126] = OBJECT_OPTION_DECLARATION_ORDER;
}

impl Default for ProcessObjectSourceOptions {
    fn default() -> Self {
        ProcessObjectSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessObjectSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ObjectSourceVisitor)
    }
}

struct ObjectSourceVisitor;

impl<'de> Visitor<'de> for ObjectSourceVisitor {
    type Value = ProcessObjectSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca process object-source options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessObjectSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca process object option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}
