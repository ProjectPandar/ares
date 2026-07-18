use crate::{
    ProjectVolumeType,
    mesh_slicer::{ChainedLayer, chain_lines_by_triangle_connectivity},
};

use super::{layers::PlannedPrintObject, raw_intersections::IntersectedPrintObject};

pub(super) struct ChainedVolumeIntersections {
    source_volume_index: usize,
    volume_ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<ChainedLayer>,
}

impl ChainedVolumeIntersections {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<ChainedLayer>) {
        (
            self.source_volume_index,
            self.volume_ordinal,
            self.volume_type,
            self.layers,
        )
    }

    #[cfg(test)]
    pub(super) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    #[cfg(test)]
    pub(super) const fn ordinal(&self) -> u32 {
        self.volume_ordinal
    }

    #[cfg(test)]
    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    #[cfg(test)]
    pub(super) fn layers(&self) -> &[ChainedLayer] {
        &self.layers
    }
}

pub(super) struct ChainedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<ChainedVolumeIntersections>,
}

impl ChainedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<ChainedVolumeIntersections>) {
        (self.plan, self.volumes)
    }

    #[cfg(test)]
    pub(super) fn plan(&self) -> &PlannedPrintObject {
        &self.plan
    }

    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[ChainedVolumeIntersections] {
        &self.volumes
    }
}

pub(super) fn chain_project_intersections(
    objects: Vec<IntersectedPrintObject>,
) -> Vec<ChainedPrintObject> {
    objects
        .into_iter()
        .map(|object| {
            let (plan, volumes) = object.into_parts();
            let volumes = volumes
                .into_iter()
                .map(|volume| {
                    let (source_volume_index, volume_ordinal, volume_type, layers) =
                        volume.into_parts();
                    let layers = layers
                        .into_iter()
                        .map(chain_lines_by_triangle_connectivity)
                        .collect();
                    ChainedVolumeIntersections {
                        source_volume_index,
                        volume_ordinal,
                        volume_type,
                        layers,
                    }
                })
                .collect();
            ChainedPrintObject { plan, volumes }
        })
        .collect()
}
