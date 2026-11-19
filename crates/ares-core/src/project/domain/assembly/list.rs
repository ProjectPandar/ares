use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::AssemblyError;

/// The byte representation of `assemble_plate_info_t` from `Format/AssembleList.hpp`.
#[derive(Debug)]
pub struct AssemblyList {
    pub(super) plates: Vec<Plate>,
}

#[derive(Deserialize)]
struct PlateListWire {
    plates: Vec<Plate>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Plate {
    pub(super) plate_name: String,
    pub(super) objects: Vec<Object>,
    need_arrange: bool,
    #[serde(default)]
    plate_params: BTreeMap<String, String>,
    #[serde(default)]
    assembled_params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Object {
    pub(super) path: String,
    pub(super) count: i32,
    pub(super) filaments: Vec<i32>,
    #[serde(default)]
    pub(super) assemble_index: Vec<i32>,
    #[serde(default)]
    pos_x: Vec<f32>,
    #[serde(default)]
    pos_y: Vec<f32>,
    #[serde(default)]
    pos_z: Vec<f32>,
    #[serde(default)]
    print_params: BTreeMap<String, String>,
    #[serde(default)]
    height_ranges: Vec<serde_json::Value>,
}

impl AssemblyList {
    pub fn from_json(bytes: &[u8]) -> Result<Self, AssemblyError> {
        let PlateListWire { plates } = serde_json::from_slice(bytes)
            .map_err(|error| AssemblyError::Config(error.to_string()))?;
        let list = Self { plates };
        if !(1..=36).contains(&list.plates.len()) {
            return Err(AssemblyError::Config(format!(
                "invalid plate count {}",
                list.plates.len()
            )));
        }
        for (index, plate) in list.plates.iter().enumerate() {
            if plate.objects.is_empty() {
                return Err(AssemblyError::Config(format!(
                    "invalid object count 0 in plate {}",
                    index + 1
                )));
            }
            for object in &plate.objects {
                object.validate()?;
            }
        }
        for plate in &list.plates {
            if plate.need_arrange
                || !plate.plate_params.is_empty()
                || !plate.assembled_params.is_empty()
            {
                return Err(AssemblyError::Invalid("assembly list requires fixed placement and no plate or assembled parameter overrides".into()));
            }
            let mut ids = BTreeSet::new();
            for object in &plate.objects {
                ids.extend(object.filaments.iter().copied());
                if !object.print_params.is_empty() || !object.height_ranges.is_empty() {
                    return Err(AssemblyError::Invalid("assembly list object parameter overrides and height ranges are unsupported".into()));
                }
            }
            if ids.len() > 1 {
                return Err(AssemblyError::Invalid(
                    "assembly list supports one filament ID per plate".into(),
                ));
            }
        }
        Ok(list)
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.plates
            .iter()
            .flat_map(|plate| plate.objects.iter().map(|object| object.path.as_str()))
    }
}

impl Object {
    fn validate(&self) -> Result<(), AssemblyError> {
        let config = |reason: &str| AssemblyError::Config(format!("object {} {reason}", self.path));
        let count = self.count as usize;
        if self.count <= 0 {
            return Err(config("has an invalid clone count"));
        }
        if self.filaments.is_empty() || self.filaments.iter().any(|&id| id < 0) {
            return Err(config("has an empty filament list or negative filament ID"));
        }
        for (key, len) in [
            ("filaments", self.filaments.len()),
            ("assemble_index", self.assemble_index.len()),
            ("pos_x", self.pos_x.len()),
            ("pos_y", self.pos_y.len()),
            ("pos_z", self.pos_z.len()),
        ] {
            if len > 1 && len != count {
                return Err(config(&format!("has invalid {key} length {len}")));
            }
        }
        if self.pos_x.len() > 1 && (self.pos_y.len() != count || self.pos_z.len() != count) {
            return Err(config(
                "requires full pos_y and pos_z arrays with per-copy pos_x",
            ));
        }
        Ok(())
    }

    pub(super) fn translation(&self, copy: usize) -> [f64; 3] {
        let translation = |index| {
            [&self.pos_x, &self.pos_y, &self.pos_z]
                .map(|values| values.get(index).copied().unwrap_or(0.0) as f64)
        };
        let first = translation(0);
        if copy == 0 {
            first
        } else {
            let next = translation(if copy < self.pos_x.len() { copy } else { 0 });
            std::array::from_fn(|axis| first[axis] + next[axis])
        }
    }
}
