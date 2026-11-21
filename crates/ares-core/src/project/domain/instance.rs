use super::{Project, ProjectInstance, Transform3d};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InstanceLabel {
    Loaded(u32),
    Allocated(u32, Option<u32>),
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
        saved_identify_id: Option<u32>,
    ) -> Self {
        Self {
            object_id: identity[0],
            instance_id: identity[1],
            label: InstanceLabel::Allocated(identity[2], saved_identify_id),
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
            InstanceLabel::Allocated(..) => None,
        }
    }

    /// Plate routing and edits retain the saved ID even when it is not a loaded label.
    pub fn saved_identify_id(&self) -> Option<u32> {
        match self.label {
            InstanceLabel::Loaded(id) => Some(id),
            InstanceLabel::Allocated(_, saved_id) => saved_id,
        }
    }

    pub(crate) fn labeled_id(&self) -> u32 {
        match self.label {
            InstanceLabel::Loaded(id) | InstanceLabel::Allocated(id, _) => id,
        }
    }

    pub fn printable(&self) -> bool {
        self.printable
    }

    /// `ModelInstance::is_printable()` (`Model.hpp:1514`): printable and inside the print volume.
    /// `ModelObject::printable` is always true here: the 3MF import sets only the instance's flag
    /// (`bbs_3mf.cpp:4397, 4429, 4448`).
    pub(crate) fn is_printable(&self) -> bool {
        self.printable && self.is_inside_print_volume()
    }

    pub(crate) fn is_inside_print_volume(&self) -> bool {
        self.print_volume_state == InstancePvs::Inside
    }

    pub fn auto_drop(&self) -> bool {
        self.auto_drop
    }

    pub fn transform(&self) -> Transform3d {
        self.transform
    }
}
