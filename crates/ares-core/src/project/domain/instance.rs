use std::collections::BTreeSet;

use crate::SliceError;

use super::{Project, ProjectInstance, Transform3d};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InstanceLabel {
    Loaded(u32),
    Allocated(u32),
}

/// `ModelInstanceEPrintVolumeState` (`Model.hpp:1408-1415` at v2.5.0-beta `345efec64`): where an
/// instance sits against the print volume. `calc_print_volume_state` never yields `Limited`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstancePvs {
    Inside,
    PartlyOutside,
    FullyOutside,
}

impl Project {
    /// `ModelInstance::update_print_volume_state`'s store (`Model.hpp:1551-1553`).
    pub(crate) fn set_instance_print_volume_state(
        &mut self,
        object: usize,
        instance: usize,
        state: InstancePvs,
    ) {
        self.objects[object].instances[instance].print_volume_state = state;
    }

    /// Apply the saved-ID filter after the plate's initial volume check (`OrcaSlicer.cpp:6580–6673`).
    pub(crate) fn apply_cli_skips(&mut self, ids: &BTreeSet<i32>) -> Result<(), SliceError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut any_inside = false;
        for instance in self
            .objects
            .iter_mut()
            .flat_map(|object| &mut object.instances)
        {
            if instance.matches_cli_skip(ids) {
                instance.printable = false;
            } else {
                match instance.print_volume_state {
                    InstancePvs::Inside => any_inside = true,
                    InstancePvs::PartlyOutside => return Err(SliceError::CliObjectPartlyOutside),
                    InstancePvs::FullyOutside => (),
                }
            }
        }
        if !any_inside {
            return Err(SliceError::NoSuitableObjectsAfterCliSkip);
        }
        Ok(())
    }
}

impl ProjectInstance {
    pub(crate) fn new(
        identity: [u32; 3],
        printable: bool,
        auto_drop: bool,
        transform: Transform3d,
    ) -> Self {
        Self {
            object_id: identity[0],
            instance_id: identity[1],
            label: InstanceLabel::Loaded(identity[2]),
            printable,
            auto_drop,
            transform,
            print_volume_state: InstancePvs::Inside,
        }
    }

    pub(crate) fn unassigned(
        identity: [u32; 3],
        printable: bool,
        auto_drop: bool,
        transform: Transform3d,
    ) -> Self {
        Self {
            object_id: identity[0],
            instance_id: identity[1],
            label: InstanceLabel::Allocated(identity[2]),
            printable,
            auto_drop,
            transform,
            print_volume_state: InstancePvs::Inside,
        }
    }

    pub fn object_id(&self) -> u32 {
        self.object_id
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    pub fn loaded_label_id(&self) -> Option<u32> {
        match self.label {
            InstanceLabel::Loaded(id) => Some(id),
            InstanceLabel::Allocated(_) => None,
        }
    }

    pub(crate) fn matches_cli_skip(&self, ids: &BTreeSet<i32>) -> bool {
        // ModelInstance::loaded_id is zero if no saved label was assigned.
        ids.contains(&(self.loaded_label_id().unwrap_or_default() as i32))
    }

    pub(crate) fn labeled_id(&self) -> u32 {
        match self.label {
            InstanceLabel::Loaded(id) | InstanceLabel::Allocated(id) => id,
        }
    }

    pub fn printable(&self) -> bool {
        self.printable
    }

    /// `ModelInstance::is_printable()` (`Model.hpp:1514`): printable and inside the print volume.
    /// `ModelObject::printable` is always true here: the 3MF import sets only the instance's flag
    /// (`bbs_3mf.cpp:4397, 4429, 4448`).
    pub(crate) fn is_printable(&self) -> bool {
        self.printable && self.print_volume_state == InstancePvs::Inside
    }

    pub fn auto_drop(&self) -> bool {
        self.auto_drop
    }

    pub fn transform(&self) -> Transform3d {
        self.transform
    }
}
