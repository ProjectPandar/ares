use super::{DynamicPrintConfig, Model, PlaterSession, SliceError};

impl PlaterSession {
    /// "Add plate" (`PartPlateList::create_plate`). In GUI mode the new plate takes its default
    /// tower position (`set_default_wipe_tower_pos_for_plate(new_index, true)`,
    /// `PartPlate.cpp:6644-6646`).
    pub fn add_plate(&mut self) -> Result<(), SliceError> {
        let mut positions = self.tower_positions();
        super::arrange_job::check_positions_for_new_plate(&positions)?;
        let new_index = self.plates.plates.len();
        self.plates.create_plate(&mut Model(&mut self.project));
        let full = DynamicPrintConfig::new(self.project.settings())?;
        self.place_new_towers(&full, &mut positions, new_index)?;
        self.wipe_tower = Some(positions);
        Ok(())
    }

    /// "Delete plate" (`PartPlateList::delete_plate`); a sole plate stays. In GUI mode the plate's
    /// tower position is erased where the vectors reach it (`PartPlate.cpp:6766-6781`).
    pub fn delete_plate(&mut self, index: usize) -> bool {
        if !self
            .plates
            .delete_plate(index, &mut Model(&mut self.project))
        {
            return false;
        }
        let mut positions = self.tower_positions();
        for values in [&mut positions.0, &mut positions.1] {
            if index < values.len() {
                values.remove(index);
            }
        }
        self.wipe_tower = Some(positions);
        // `PartPlate.cpp:6809-6815`: the current plate moves back past a deleted one at or
        // before it.
        if !(self.current_plate == index && index == 0) && self.current_plate >= index {
            self.current_plate -= 1;
        }
        true
    }
}
