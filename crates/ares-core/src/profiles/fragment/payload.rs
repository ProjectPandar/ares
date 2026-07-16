use std::fmt;

use serde::de::{DeserializeSeed, IgnoredAny};

use super::{ProfileKind, metadata::is_metadata_key};
use crate::{
    SliceError,
    options::{FilamentOptionsBuilder, PrinterOptionsBuilder, ProcessOptionsBuilder},
};

#[derive(Clone, PartialEq)]
pub(in crate::profiles) enum ProfilePayload {
    Machine(Box<PrinterOptionsBuilder>),
    Process(Box<ProcessOptionsBuilder>),
    Filament(Box<FilamentOptionsBuilder>),
}

impl fmt::Debug for ProfilePayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Machine(_) => "Machine(..)",
            Self::Process(_) => "Process(..)",
            Self::Filament(_) => "Filament(..)",
        })
    }
}

pub(super) fn parse(input: &[u8], kind: ProfileKind) -> Result<ProfilePayload, SliceError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let payload = PayloadSeed { kind }
        .deserialize(&mut deserializer)
        .map_err(invalid_option)?;
    deserializer.end().map_err(invalid_option)?;
    Ok(payload)
}

fn invalid_option(error: serde_json::Error) -> SliceError {
    SliceError::InvalidInput(format!("profile option JSON is invalid: {error}"))
}

struct PayloadSeed {
    kind: ProfileKind,
}

impl<'de> DeserializeSeed<'de> for PayloadSeed {
    type Value = ProfilePayload;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(PayloadVisitor {
            builder: match self.kind {
                ProfileKind::Machine => ProfilePayload::Machine(Box::default()),
                ProfileKind::Process => ProfilePayload::Process(Box::default()),
                ProfileKind::Filament => ProfilePayload::Filament(Box::default()),
            },
        })
    }
}

struct PayloadVisitor {
    builder: ProfilePayload,
}

impl<'de> serde::de::Visitor<'de> for PayloadVisitor {
    type Value = ProfilePayload;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an Orca profile option object")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<String>()? {
            if is_metadata_key(&key) {
                map.next_value::<IgnoredAny>()?;
                continue;
            }
            let known = match &mut self.builder {
                ProfilePayload::Machine(builder) => {
                    builder.deserialize_known_field(&key, &mut map)?
                }
                ProfilePayload::Process(builder) => {
                    builder.deserialize_known_field(&key, &mut map)?
                }
                ProfilePayload::Filament(builder) => {
                    builder.deserialize_known_field(&key, &mut map)?
                }
            };
            if !known {
                return Err(serde::de::Error::custom(format_args!(
                    "unknown or misplaced profile option {key}"
                )));
            }
        }
        Ok(self.builder)
    }
}
