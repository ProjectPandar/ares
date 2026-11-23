use crate::{SliceError, profiles::VendorType};
use serde::{Deserialize, Serialize};

use super::PlaterSession;

const MAX_SLOTS: usize = 64; // libslic3r.h:70
const ADD_COLOURS: [&str; 16] = [
    "#00C1AE", "#F4E2C1", "#ED1C24", "#00FF7F", "#F26722", "#FFEB31", "#7841CE", "#115877",
    "#ED1E79", "#2EBDEF", "#345B2F", "#800080", "#FA8173", "#800000", "#F7B763", "#A4C41E",
];

/// An edit of the ordered project palette. Slot IDs are 1-based, in the order before the edit.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilamentSlotEdit {
    Add,
    Delete {
        slot: i32,
    },
    Merge {
        source: i32,
        target: i32,
        #[serde(default)]
        confirmed: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilamentSlot {
    pub preset: String,
    pub colour: String,
    pub is_mixed: bool,
    pub components: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilamentSlots {
    pub slots: Vec<FilamentSlot>,
    pub can_add: bool,
    pub can_remove: bool,
    pub edited: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SlotChange {
    Insert { at: i32, colour: &'static str },
    Remove { at: i32, replacement: Option<i32> },
}

impl SlotChange {
    pub(crate) fn surviving(self, slot: i32) -> Option<i32> {
        match self {
            Self::Insert { at, .. } => Some(slot + i32::from(slot >= at)),
            Self::Remove { at, .. } if slot == at => None,
            Self::Remove { at, .. } => Some(slot - i32::from(slot > at)),
        }
    }

    pub(crate) fn assignment(self, slot: i32, default: i32) -> i32 {
        self.surviving(slot).unwrap_or(match self {
            Self::Remove { replacement, .. } => replacement.unwrap_or(default),
            Self::Insert { .. } => default,
        })
    }

    pub(crate) fn previous_index(self, index: usize) -> Option<usize> {
        let slot = index as i32 + 1;
        match self {
            Self::Insert { at, .. } if at == slot => None,
            Self::Insert { at, .. } => Some(index - usize::from(slot > at)),
            Self::Remove { at, .. } => Some(index + usize::from(slot >= at)),
        }
    }
}

impl PlaterSession {
    pub fn filament_slots(&self) -> FilamentSlots {
        let slots = self.project.filament_slots();
        let editable = self
            .project
            .settings()
            .printer
            .gcode
            .single_extruder_multi_material
            .0
            || self.vendor_type == VendorType::MarlinBbl;
        let physical = slots.iter().filter(|slot| !slot.is_mixed).count();
        FilamentSlots {
            can_add: editable && slots.len() < MAX_SLOTS,
            can_remove: editable && physical > 1,
            edited: !self.slot_changes.is_empty(),
            slots,
        }
    }

    pub fn edit_filament_slots(
        &mut self,
        edit: FilamentSlotEdit,
    ) -> Result<FilamentSlots, SliceError> {
        let view = self.filament_slots();
        let physical = |slot: i32| {
            slot > 0
                && view
                    .slots
                    .get(slot as usize - 1)
                    .is_some_and(|slot| !slot.is_mixed)
        };
        let change = match edit {
            FilamentSlotEdit::Add if view.can_add => SlotChange::Insert {
                at: view.slots.iter().filter(|slot| !slot.is_mixed).count() as i32 + 1,
                colour: ADD_COLOURS[self.next_filament_colour % ADD_COLOURS.len()],
            },
            FilamentSlotEdit::Delete { slot } if view.can_remove && physical(slot) => {
                SlotChange::Remove {
                    at: slot,
                    replacement: None,
                }
            }
            FilamentSlotEdit::Merge {
                source,
                target,
                confirmed,
            } if view.can_remove
                && physical(source)
                && source != target
                && (1..=view.slots.len() as i32).contains(&target) =>
            {
                // Plater.cpp:5897–5929 warns before removing a component of the target.
                if !confirmed && view.slots[target as usize - 1].components.contains(&source) {
                    return Err(SliceError::InvalidInput(
                        "Merging would remove a component of the target mixed filament".into(),
                    ));
                }
                SlotChange::Remove {
                    at: source,
                    replacement: Some(target - i32::from(target > source)),
                }
            }
            _ => {
                return Err(SliceError::InvalidInput(
                    "Filament slot edit is not available".into(),
                ));
            }
        };
        self.project.apply_slot_options(change)?;
        self.project.remap_slot_references(change);
        for slot in self.filament_edits.values_mut().flatten() {
            *slot = change.assignment(*slot, 1);
        }
        self.slot_changes.push(change);
        if matches!(edit, FilamentSlotEdit::Add) {
            self.next_filament_colour += 1;
        }
        Ok(self.filament_slots())
    }
}
