use super::PlaterSession;
use crate::SliceError;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Stable ids in the session scene. An instance edit targets its owning object.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilamentTarget {
    pub object: u32,
    pub volume: Option<u32>,
}

pub(crate) type FilamentEdits = BTreeMap<FilamentTarget, Option<i32>>;

fn validate_slot(slot: i32, count: i32) -> Result<(), SliceError> {
    if !(0..=count).contains(&slot) {
        return Err(SliceError::InvalidInput(
            "Filament slot is out of range".into(),
        ));
    }
    Ok(())
}

impl PlaterSession {
    /// Applies a Prepare filament choice to the targets in selection order.
    /// Slot zero is Default; the slot count is from the active project or preset selection.
    pub fn set_filament(
        &mut self,
        targets: &[FilamentTarget],
        (slot, count): (i32, i32),
    ) -> Result<(), SliceError> {
        validate_slot(slot, count)?;
        self.project
            .change_filament(targets, slot, &mut self.filament_edits);
        Ok(())
    }

    /// Applies an inline choice to one row. An object choice clears every volume override;
    /// slot zero is only a choice for a parameter modifier.
    pub fn set_filament_cell(
        &mut self,
        target: FilamentTarget,
        (slot, count): (i32, i32),
    ) -> Result<(), SliceError> {
        validate_slot(slot, count)?;
        self.project
            .change_filament_cell(target, slot, &mut self.filament_edits);
        Ok(())
    }
}
