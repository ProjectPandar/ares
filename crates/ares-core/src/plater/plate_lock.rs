//! The plate lock (`PartPlate::is_locked` / `lock`, `PartPlate.hpp:614-615`, and
//! `PartPlateList::lock_plate`, `PartPlate.cpp:7160-7176` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-plate-lock.md`.

use super::PlaterSession;
use super::plate_list::Plate;
use crate::SliceError;

impl PlaterSession {
    /// `PartPlateList::lock_plate`: locks or unlocks the plate at the 0-based `plate`.
    pub fn lock_plate(&mut self, plate: usize, locked: bool) -> Result<(), SliceError> {
        self.plate_at(plate)?;
        self.plates.plates[plate].locked = locked;
        Ok(())
    }

    /// `PartPlateList::is_locked` (`PartPlate.hpp:1010`).
    pub fn is_plate_locked(&self, plate: usize) -> Result<bool, SliceError> {
        Ok(self.plate_at(plate)?.locked)
    }

    /// The plate at `plate`; upstream's `get_plate` returns null past the list.
    fn plate_at(&self, plate: usize) -> Result<&Plate, SliceError> {
        self.plates
            .plates
            .get(plate)
            .ok_or_else(|| SliceError::InvalidInput(format!("there is no plate {plate}")))
    }
}
