//! The plate list's current plate (`PartPlateList::select_plate`, `PartPlate.cpp:6958-7007` at
//! v2.5.0-beta `345efec64`). See `docs/architecture/web-plater-arrange-object-skirt.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// `PartPlateList::select_plate`: makes the 0-based `plate` the current plate.
    pub fn select_plate(&mut self, plate: usize) -> Result<(), SliceError> {
        if plate >= self.plates.plates.len() {
            return Err(SliceError::InvalidInput(format!(
                "there is no plate {plate}"
            )));
        }
        self.current_plate = plate;
        Ok(())
    }

    /// `PartPlateList::get_curr_plate_index`.
    pub fn current_plate(&self) -> usize {
        self.current_plate
    }
}
