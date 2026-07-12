#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePaintedSortParent {
    print_object_region_id: u64,
}

impl StagedGeneratePaintedSortParent {
    pub(super) fn new(print_object_region_id: u64) -> Self {
        Self {
            print_object_region_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePaintedSortableRegion {
    extruder_id: u32,
    parent: usize,
    region_id: u64,
    marker: u64,
}

impl StagedGeneratePaintedSortableRegion {
    pub(super) fn new(extruder_id: u32, parent: usize, region_id: u64, marker: u64) -> Self {
        Self {
            extruder_id,
            parent,
            region_id,
            marker,
        }
    }

    pub(super) fn extruder_id(&self) -> u32 {
        self.extruder_id
    }

    pub(super) fn parent(&self) -> usize {
        self.parent
    }

    pub(super) fn region_id(&self) -> u64 {
        self.region_id
    }

    pub(super) fn marker(&self) -> u64 {
        self.marker
    }
}

pub(super) fn staged_sort_generate_painted_regions(
    parent_regions: &[StagedGeneratePaintedSortParent],
    painted_regions: &[StagedGeneratePaintedSortableRegion],
) -> Vec<StagedGeneratePaintedSortableRegion> {
    let mut sorted = painted_regions.to_vec();
    sorted.sort_by_key(|region| {
        (
            parent_regions[region.parent].print_object_region_id,
            region.extruder_id,
        )
    });
    sorted
}
