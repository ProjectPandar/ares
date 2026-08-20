use crate::{
    ProjectVolumeType, RegionOptions,
    geometry::{ExPolygon, Polygon},
};

pub(super) mod complex;
#[cfg(test)]
mod tests;

use super::{
    layers::PlannedPrintObject,
    volume_bounds::{BoundedPrintObject, PostBoundsVolume, VolumeBound, VolumeOccurrenceId},
    volume_regions::{VolumeRegion, VolumeRegionGraph},
};

#[derive(Clone)]
pub(super) struct VolumeSlices {
    pub(super) occurrence_id: VolumeOccurrenceId,
    pub(super) layers: Vec<Vec<ExPolygon>>,
}

impl VolumeSlices {
    #[cfg(any(test, feature = "task22n-browser-oracle"))]
    pub(super) fn as_parts(&self) -> (VolumeOccurrenceId, &[Vec<ExPolygon>]) {
        (self.occurrence_id, &self.layers)
    }
}

pub(super) struct PostRegionPrintObject {
    pub(super) plan: PlannedPrintObject,
    pub(super) volume_slices: Vec<VolumeSlices>,
    pub(super) regions: Vec<PostRegion>,
}

impl PostRegionPrintObject {
    pub(super) fn as_parts(&self) -> (&PlannedPrintObject, &[VolumeSlices], &[PostRegion]) {
        (&self.plan, &self.volume_slices, &self.regions)
    }

    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<VolumeSlices>, Vec<PostRegion>) {
        (self.plan, self.volume_slices, self.regions)
    }
}

pub(super) struct PostRegion {
    pub(super) id: usize,
    pub(super) options: RegionOptions,
    pub(super) layers: Vec<RegionLayer>,
}

impl PostRegion {
    pub(super) fn as_parts(&self) -> (usize, &RegionOptions, &[RegionLayer]) {
        (self.id, &self.options, &self.layers)
    }

    #[cfg(test)]
    pub(super) fn into_parts(self) -> (usize, RegionOptions, Vec<RegionLayer>) {
        (self.id, self.options, self.layers)
    }
}

pub(super) struct RegionLayer {
    pub(super) surfaces: Vec<RegionSurface>,
}

impl RegionLayer {
    pub(super) fn surfaces(&self) -> &[RegionSurface] {
        &self.surfaces
    }

    #[cfg(test)]
    pub(super) fn into_parts(self) -> Vec<RegionSurface> {
        self.surfaces
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum RegionSurfaceKind {
    Top = 0,
    Bottom = 1,
    BottomBridge = 2,
    Internal = 4,
    InternalSolid = 5,
    InternalBridge = 6,
    InternalVoid = 8,
}

impl RegionSurfaceKind {
    pub(super) const fn is_bridge(self) -> bool {
        match self {
            Self::BottomBridge | Self::InternalBridge => true,
            Self::Top
            | Self::Bottom
            | Self::Internal
            | Self::InternalSolid
            | Self::InternalVoid => false,
        }
    }
}

#[derive(Clone)]
pub(super) struct RegionSurface {
    kind: RegionSurfaceKind,
    expolygon: ExPolygon,
    thickness: f64,
    thickness_layers: u16,
    bridge_angle: f64,
    extra_perimeters: u16,
}

impl RegionSurface {
    pub(super) fn new(kind: RegionSurfaceKind, expolygon: ExPolygon) -> Self {
        Self {
            kind,
            expolygon,
            thickness: -1.0,
            thickness_layers: 1,
            bridge_angle: -1.0,
            extra_perimeters: 0,
        }
    }

    pub(super) fn internal(expolygon: ExPolygon) -> Self {
        Self::new(RegionSurfaceKind::Internal, expolygon)
    }

    pub(super) fn clone_with_kind(&self, kind: RegionSurfaceKind) -> Self {
        let mut surface = self.clone();
        surface.kind = kind;
        surface
    }

    pub(super) fn clone_with_expolygon(&self, expolygon: ExPolygon) -> Self {
        let mut surface = self.clone();
        surface.expolygon = expolygon;
        surface
    }

    pub(super) fn retag(&mut self, kind: RegionSurfaceKind) {
        self.kind = kind;
    }

    pub(super) fn take_expolygon(&mut self) -> ExPolygon {
        std::mem::replace(
            &mut self.expolygon,
            ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
        )
    }

    pub(super) fn set_bridge_angle(&mut self, bridge_angle: f64) {
        self.bridge_angle = bridge_angle;
    }
    pub(super) fn set_thickness(&mut self, thickness: f64) {
        self.thickness = thickness;
    }

    #[cfg(test)]
    pub(super) fn internal_with_metadata(
        expolygon: ExPolygon,
        thickness: f64,
        thickness_layers: u16,
        bridge_angle: f64,
        extra_perimeters: u16,
    ) -> Self {
        Self {
            kind: RegionSurfaceKind::Internal,
            expolygon,
            thickness,
            thickness_layers,
            bridge_angle,
            extra_perimeters,
        }
    }

    pub(super) fn as_parts(&self) -> (RegionSurfaceKind, &ExPolygon, f64, u16, f64, u16) {
        (
            self.kind,
            &self.expolygon,
            self.thickness,
            self.thickness_layers,
            self.bridge_angle,
            self.extra_perimeters,
        )
    }

    pub(super) fn into_parts(self) -> (RegionSurfaceKind, ExPolygon, f64, u16, f64, u16) {
        (
            self.kind,
            self.expolygon,
            self.thickness,
            self.thickness_layers,
            self.bridge_angle,
            self.extra_perimeters,
        )
    }
}

pub(super) struct PendingRegionSlices {
    output: PostRegionPrintObject,
    working_slices: Vec<VolumeSlices>,
    bounds: Vec<VolumeBound>,
    volume_regions: Vec<VolumeRegion>,
    complex_layer_indices: Vec<usize>,
}

impl PendingRegionSlices {
    #[cfg(test)]
    pub(super) fn as_parts(
        &self,
    ) -> (
        &PostRegionPrintObject,
        &[VolumeSlices],
        &[VolumeBound],
        &[VolumeRegion],
        &[usize],
    ) {
        (
            &self.output,
            &self.working_slices,
            &self.bounds,
            &self.volume_regions,
            &self.complex_layer_indices,
        )
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        PostRegionPrintObject,
        Vec<VolumeSlices>,
        Vec<VolumeBound>,
        Vec<VolumeRegion>,
        Vec<usize>,
    ) {
        (
            self.output,
            self.working_slices,
            self.bounds,
            self.volume_regions,
            self.complex_layer_indices,
        )
    }
}

pub(super) fn prepare_region_slices(
    bounded: BoundedPrintObject,
    graph: VolumeRegionGraph,
) -> PendingRegionSlices {
    let (plan, volumes, bounds) = bounded.into_parts();
    let mut working_slices = volumes.into_iter().map(convert_volume).collect::<Vec<_>>();
    working_slices.sort_by_key(|volume| volume.occurrence_id);
    let volume_slices = working_slices.clone();
    let VolumeRegionGraph {
        all_regions,
        volume_regions,
    } = graph;
    let layer_count = plan.layers.len();
    let regions = all_regions
        .into_iter()
        .enumerate()
        .map(|(id, options)| PostRegion {
            id,
            options,
            layers: (0..layer_count)
                .map(|_| RegionLayer {
                    surfaces: Vec::new(),
                })
                .collect(),
        })
        .collect();
    let mut output = PostRegionPrintObject {
        plan,
        volume_slices,
        regions,
    };
    let mut complex_layer_indices = Vec::new();

    match volume_regions.as_slice() {
        [] => {}
        [record] => {
            if record.kind == ProjectVolumeType::ModelPart {
                for layer_index in 0..layer_count {
                    move_layer(&mut output, &mut working_slices, record, layer_index);
                }
            }
        }
        records => {
            for layer_index in 0..layer_count {
                let z = output.plan.layers[layer_index].slice_z as f32;
                let active =
                    |record: &VolumeRegion| bounds[record.bound_index].bbox().contains_z(z);
                let Some(first_index) = records
                    .iter()
                    .enumerate()
                    .find(|record| {
                        record.1.kind == ProjectVolumeType::ModelPart && active(record.1)
                    })
                    .map(|(index, _)| index)
                else {
                    continue;
                };
                let complex = (first_index + 1..records.len()).any(|record_index| {
                    let record = &records[record_index];
                    active(record)
                        && overlaps_active_predecessor(
                            records,
                            &bounds,
                            first_index,
                            record_index,
                            z,
                        )
                });
                if complex {
                    complex_layer_indices.push(layer_index);
                } else {
                    move_layer(
                        &mut output,
                        &mut working_slices,
                        &records[first_index],
                        layer_index,
                    );
                }
            }
        }
    }

    PendingRegionSlices {
        output,
        working_slices,
        bounds,
        volume_regions,
        complex_layer_indices,
    }
}

fn convert_volume(volume: PostBoundsVolume) -> VolumeSlices {
    let (_, occurrence_id, _, layers) = volume.into_parts();
    VolumeSlices {
        occurrence_id,
        layers: layers
            .into_iter()
            .map(|layer| layer.into_parts().1)
            .collect(),
    }
}

fn move_layer(
    output: &mut PostRegionPrintObject,
    working_slices: &mut [VolumeSlices],
    record: &VolumeRegion,
    layer_index: usize,
) {
    let volume = volume_by_occurrence_mut(working_slices, record.occurrence_id);
    let expolygons = std::mem::take(&mut volume.layers[layer_index]);
    let region_id = record
        .region_id
        .expect("printable record must own a region");
    output.regions[region_id].layers[layer_index]
        .surfaces
        .extend(expolygons.into_iter().map(RegionSurface::internal));
}

fn volume_by_occurrence_mut(
    volumes: &mut [VolumeSlices],
    occurrence_id: VolumeOccurrenceId,
) -> &mut VolumeSlices {
    let index = volumes
        .binary_search_by_key(&occurrence_id, |volume| volume.occurrence_id)
        .expect("region record must retain a physical carrier");
    &mut volumes[index]
}

fn overlaps_active_predecessor(
    records: &[VolumeRegion],
    bounds: &[VolumeBound],
    first_index: usize,
    record_index: usize,
    z: f32,
) -> bool {
    let current = bounds[records[record_index].bound_index].bbox();
    records[first_index..record_index].iter().any(|record| {
        let predecessor = bounds[record.bound_index].bbox();
        predecessor.contains_z(z) && current.intersects_xy(predecessor)
    })
}
