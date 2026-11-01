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
}

pub(super) struct ChainedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<ChainedVolumeIntersections>,
}

impl ChainedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<ChainedVolumeIntersections>) {
        (self.plan, self.volumes)
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
                    let (source_volume_index, volume_ordinal, volume_type, layers, coplanar_edges) =
                        volume.into_parts();
                    let layers = layers
                        .into_iter()
                        .map(|lines| chain_lines_by_triangle_connectivity(lines, &coplanar_edges))
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
