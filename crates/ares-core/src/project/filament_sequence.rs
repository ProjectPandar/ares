use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::SliceError;

use super::xml::{JsonRole, deserialize_json};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FilamentSequences(pub BTreeMap<PlateId, PlateFilamentSequence>);

impl FilamentSequences {
    pub(crate) fn read(
        bytes: &[u8],
        plate_ids: impl Iterator<Item = u32>,
    ) -> Result<Self, SliceError> {
        let document: Value = deserialize_json(bytes, JsonRole::FilamentSequences)?;
        let mut sequences = BTreeMap::new();
        for id in plate_ids {
            let Some(plate) = document
                .get(format!("plate_{id}"))
                .and_then(Value::as_object)
            else {
                continue;
            };
            let filament = plate
                .get("filament_sequence")
                .or_else(|| plate.get("sequence"));
            let nozzle = plate.get("nozzle_sequence");
            let (Some(filament), Some(nozzle)) = (filament, nozzle) else {
                continue;
            };
            if !filament.is_array() || !nozzle.is_array() {
                continue;
            }
            let parse = || -> Result<PlateFilamentSequence, serde_json::Error> {
                let sequence = Vec::<u32>::deserialize(filament)?
                    .into_iter()
                    .map(|id| id.saturating_sub(1))
                    .collect();
                let nozzle_sequence = Vec::<u32>::deserialize(nozzle)?;
                let optimal_assignment = match plate.get("optimal_assignment") {
                    Some(value) if value.is_array() => Vec::<i32>::deserialize(value)?,
                    _ => Vec::new(),
                };
                Ok(PlateFilamentSequence {
                    sequence,
                    nozzle_sequence,
                    optimal_assignment,
                })
            };
            let sequence = parse().map_err(|error| {
                SliceError::InvalidInput(format!(
                    "invalid project filament sequences JSON: {error}"
                ))
            })?;
            sequences.insert(PlateId::new(id), sequence);
        }
        Ok(Self(sequences))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PlateId(u32);

impl PlateId {
    pub(crate) const fn new(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlateFilamentSequence {
    pub sequence: Vec<u32>,
    pub nozzle_sequence: Vec<u32>,
    pub optimal_assignment: Vec<i32>,
}
