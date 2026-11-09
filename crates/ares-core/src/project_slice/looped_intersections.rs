use crate::{ProjectVolumeType, mesh_slicer::LoopedLayer};

use super::layers::PlannedPrintObject;

pub(super) struct LoopedVolumeIntersections {
    pub(super) source_volume_index: usize,
    pub(super) volume_ordinal: u32,
    pub(super) volume_type: ProjectVolumeType,
    pub(super) layers: Vec<LoopedLayer>,
}

impl LoopedVolumeIntersections {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<LoopedLayer>) {
        (
            self.source_volume_index,
            self.volume_ordinal,
            self.volume_type,
            self.layers,
        )
    }
}

pub(super) struct LoopedPrintObject {
    pub(super) plan: PlannedPrintObject,
    pub(super) volumes: Vec<LoopedVolumeIntersections>,
}

impl LoopedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<LoopedVolumeIntersections>) {
        (self.plan, self.volumes)
    }
}
