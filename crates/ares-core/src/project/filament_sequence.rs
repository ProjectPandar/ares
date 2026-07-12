use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Deserializer, de::Visitor};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct FilamentSequences(pub BTreeMap<PlateId, PlateFilamentSequence>);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PlateId(u32);

impl PlateId {
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for PlateId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PlateIdVisitor)
    }
}

struct PlateIdVisitor;

impl Visitor<'_> for PlateIdVisitor {
    type Value = PlateId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical plate_<positive u32> member name")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let digits = value
            .strip_prefix("plate_")
            .filter(|digits| !digits.is_empty())
            .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Str(value), &self))?;
        if !digits.bytes().all(|byte| byte.is_ascii_digit())
            || (digits.len() > 1 && digits.starts_with('0'))
        {
            return Err(E::invalid_value(serde::de::Unexpected::Str(value), &self));
        }
        let id = digits
            .parse::<u32>()
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Str(value), &self))?;
        Ok(PlateId(id))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PlateFilamentSequence {
    pub sequence: Vec<u32>,
    pub nozzle_sequence: Vec<u32>,
    #[serde(default)]
    pub optimal_assignment: Vec<i32>,
}
