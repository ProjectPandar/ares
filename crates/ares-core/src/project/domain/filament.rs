use super::{Project, ProjectObject, ProjectVolume, ProjectVolumeType};
use crate::{FilamentTarget, OrcaInt, plater::FilamentEdits};

impl ProjectObject {
    /// The explicitly stored filament slot, if present in the object config.
    pub fn filament(&self) -> Option<i32> {
        self.region_overrides.extruder.map(|slot| slot.0)
    }

    fn change_filament(&mut self, volume_id: Option<u32>, slot: i32, edits: &mut FilamentEdits) {
        let target = FilamentTarget {
            object: self.id,
            volume: volume_id,
        };
        if let Some(id) = volume_id {
            let parent = self.filament().unwrap_or(1);
            let Some(volume) = self.volumes.iter_mut().find(|volume| volume.id == id) else {
                return;
            };
            let selected = match volume.volume_type {
                ProjectVolumeType::ModelPart if slot == 0 => parent,
                ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier => slot,
                _ => return,
            };
            volume.region_overrides.extruder = Some(OrcaInt(selected));
            edits.insert(target, Some(selected));
            return;
        }
        let selected = slot.max(1);
        self.region_overrides.extruder = Some(OrcaInt(selected));
        edits.insert(target, Some(selected));
        for volume in self
            .volumes
            .iter_mut()
            .filter(|v| v.volume_type == ProjectVolumeType::ModelPart)
        {
            volume.region_overrides.extruder = None;
            edits.insert(
                FilamentTarget {
                    volume: Some(volume.id),
                    ..target
                },
                None,
            );
        }
    }
}

impl ProjectVolume {
    /// The explicitly stored filament slot, including zero for Default.
    pub fn filament(&self) -> Option<i32> {
        self.region_overrides.extruder.map(|slot| slot.0)
    }

    /// ModelVolume::extruder_id (Model.cpp:2690): the slot used for Prepare colours.
    pub fn effective_filament(&self, object: &ProjectObject) -> i32 {
        self.filament()
            .filter(|&slot| slot != 0)
            .or_else(|| object.filament())
            .unwrap_or(1)
    }
}

impl Project {
    pub(crate) fn change_filament(
        &mut self,
        targets: &[FilamentTarget],
        slot: i32,
        edits: &mut FilamentEdits,
    ) {
        // ObjectList::set_extruder_for_selected_items (GUI_ObjectList.cpp:6910–6974).
        for target in targets {
            if let Some(object) = self
                .objects
                .iter_mut()
                .find(|object| object.id == target.object)
            {
                object.change_filament(target.volume, slot, edits);
            }
        }
    }

    pub(super) fn replay_filament_edits(&mut self, edits: &FilamentEdits) {
        for (&target, &slot) in edits {
            let Some(object) = self
                .objects
                .iter_mut()
                .find(|object| object.id == target.object)
            else {
                continue;
            };
            let config = match target.volume {
                None => Some(&mut object.region_overrides),
                Some(id) => object
                    .volumes
                    .iter_mut()
                    .find(|volume| volume.id == id)
                    .map(|volume| &mut volume.region_overrides),
            };
            if let Some(config) = config {
                config.extruder = slot.map(OrcaInt);
            }
        }
    }
}
