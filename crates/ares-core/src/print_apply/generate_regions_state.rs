use super::mesh_state::StagedLayerHeightRange;
use super::volume_cache_state::StagedVolumeExtents;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateLayerRange {
    layer_height_range: StagedLayerHeightRange,
    config_id: Option<u64>,
}

impl StagedGenerateLayerRange {
    pub(super) fn new(layer_height_range: StagedLayerHeightRange, config_id: Option<u64>) -> Self {
        Self {
            layer_height_range,
            config_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedGeneratePrintObjectRegionsLayer {
    layer_height_range: StagedLayerHeightRange,
    config_id: Option<u64>,
    volumes: Vec<StagedVolumeExtents>,
    volume_regions: Vec<u64>,
    painted_regions: Vec<u64>,
    fuzzy_skin_painted_regions: Vec<u64>,
}

impl StagedGeneratePrintObjectRegionsLayer {
    pub(super) fn new(
        layer_height_range: StagedLayerHeightRange,
        config_id: Option<u64>,
        volumes: Vec<StagedVolumeExtents>,
        region_lists: (Vec<u64>, Vec<u64>, Vec<u64>),
    ) -> Self {
        let (volume_regions, painted_regions, fuzzy_skin_painted_regions) = region_lists;
        Self {
            layer_height_range,
            config_id,
            volumes,
            volume_regions,
            painted_regions,
            fuzzy_skin_painted_regions,
        }
    }

    pub(super) fn layer_height_range(&self) -> StagedLayerHeightRange {
        self.layer_height_range
    }

    pub(super) fn config_id(&self) -> Option<u64> {
        self.config_id
    }

    pub(super) fn volumes(&self) -> &[StagedVolumeExtents] {
        &self.volumes
    }

    pub(super) fn volume_regions(&self) -> &[u64] {
        &self.volume_regions
    }

    pub(super) fn painted_regions(&self) -> &[u64] {
        &self.painted_regions
    }

    pub(super) fn fuzzy_skin_painted_regions(&self) -> &[u64] {
        &self.fuzzy_skin_painted_regions
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateModelVolumePaint {
    id: u64,
    is_mm_painted: bool,
}

impl StagedGenerateModelVolumePaint {
    pub(super) fn new(id: u64, is_mm_painted: bool) -> Self {
        Self { id, is_mm_painted }
    }

    pub(super) fn is_mm_painted(&self) -> bool {
        self.is_mm_painted
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedUpdateVolumeBboxesCall {
    is_mm_painted: bool,
    offset: f32,
    trafo_bboxes: u64,
    cached_volume_ids: Vec<u64>,
    layer_range_count: usize,
}

impl StagedUpdateVolumeBboxesCall {
    pub(super) fn is_mm_painted(&self) -> bool {
        self.is_mm_painted
    }

    pub(super) fn offset(&self) -> f32 {
        self.offset
    }

    pub(super) fn trafo_bboxes(&self) -> u64 {
        self.trafo_bboxes
    }

    pub(super) fn cached_volume_ids(&self) -> &[u64] {
        &self.cached_volume_ids
    }

    pub(super) fn layer_range_count(&self) -> usize {
        self.layer_range_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct StagedGenerateRegionConfigKey {
    hash: u64,
    ordinal: u64,
}

impl StagedGenerateRegionConfigKey {
    pub(super) fn new(hash: u64, ordinal: u64) -> Self {
        Self { hash, ordinal }
    }

    pub(super) fn hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePrintRegion {
    id: u64,
    config: StagedGenerateRegionConfigKey,
}

impl StagedGeneratePrintRegion {
    pub(super) fn new(id: u64, config: StagedGenerateRegionConfigKey) -> Self {
        Self { id, config }
    }

    pub(super) fn id(&self) -> u64 {
        self.id
    }

    pub(super) fn config_hash(&self) -> u64 {
        self.config.hash()
    }

    pub(super) fn config(&self) -> StagedGenerateRegionConfigKey {
        self.config
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedGeneratePrintObjectRegions {
    all_regions: Vec<StagedGeneratePrintRegion>,
    layer_ranges: Vec<StagedGeneratePrintObjectRegionsLayer>,
    trafo_bboxes: u64,
    cached_volume_ids: Vec<u64>,
}

impl StagedGeneratePrintObjectRegions {
    pub(super) fn new(
        all_regions: Vec<StagedGeneratePrintRegion>,
        layer_ranges: Vec<StagedGeneratePrintObjectRegionsLayer>,
        trafo_bboxes: u64,
        cached_volume_ids: Vec<u64>,
    ) -> Self {
        Self {
            all_regions,
            layer_ranges,
            trafo_bboxes,
            cached_volume_ids,
        }
    }

    pub(super) fn all_regions(&self) -> &[StagedGeneratePrintRegion] {
        &self.all_regions
    }

    pub(super) fn layer_ranges(&self) -> &[StagedGeneratePrintObjectRegionsLayer] {
        &self.layer_ranges
    }

    pub(super) fn trafo_bboxes(&self) -> u64 {
        self.trafo_bboxes
    }

    pub(super) fn cached_volume_ids(&self) -> &[u64] {
        &self.cached_volume_ids
    }
}

pub(super) fn staged_generate_print_object_regions_layer_range_shell(
    old: Option<StagedGeneratePrintObjectRegions>,
    model_layer_ranges: &[StagedGenerateLayerRange],
    trafo_bboxes: u64,
) -> StagedGeneratePrintObjectRegions {
    let mut out = old.unwrap_or_else(|| {
        StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), trafo_bboxes, Vec::new())
    });
    out.all_regions.clear();

    if !out.layer_ranges.is_empty() {
        assert_eq!(model_layer_ranges.len(), out.layer_ranges.len());
        for (range, layer_range) in model_layer_ranges.iter().zip(&mut out.layer_ranges) {
            assert_eq!(range.layer_height_range, layer_range.layer_height_range);
            layer_range.config_id = range.config_id;
            layer_range.volume_regions.clear();
            layer_range.painted_regions.clear();
            layer_range.fuzzy_skin_painted_regions.clear();
        }
    } else {
        out.trafo_bboxes = trafo_bboxes;
        out.layer_ranges.reserve(model_layer_ranges.len());
        for range in model_layer_ranges {
            out.layer_ranges
                .push(StagedGeneratePrintObjectRegionsLayer::new(
                    range.layer_height_range,
                    range.config_id,
                    Vec::new(),
                    (Vec::new(), Vec::new(), Vec::new()),
                ));
        }
    }

    out
}

pub(super) fn staged_generate_print_object_regions_update_volume_bboxes_call(
    shell: &StagedGeneratePrintObjectRegions,
    model_volumes: &[StagedGenerateModelVolumePaint],
    num_extruders: usize,
    xy_contour_compensation: f32,
) -> StagedUpdateVolumeBboxesCall {
    let is_mm_painted =
        num_extruders > 1 && model_volumes.iter().any(|volume| volume.is_mm_painted());
    let offset = if is_mm_painted {
        0.0
    } else {
        xy_contour_compensation.max(0.0)
    };

    StagedUpdateVolumeBboxesCall {
        is_mm_painted,
        offset,
        trafo_bboxes: shell.trafo_bboxes,
        cached_volume_ids: shell.cached_volume_ids.clone(),
        layer_range_count: shell.layer_ranges.len(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct StagedGenerateRegionSet {
    lookup_region_ids: Vec<u64>,
}

impl StagedGenerateRegionSet {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn get_create_region(
        &mut self,
        shell: &mut StagedGeneratePrintObjectRegions,
        config: StagedGenerateRegionConfigKey,
    ) -> u64 {
        let hash = config.hash();
        let insertion_index = self.lookup_region_ids.partition_point(|region_id| {
            let region = shell
                .all_regions
                .iter()
                .find(|region| region.id() == *region_id)
                .unwrap();
            region.config_hash() < hash
                || (region.config_hash() == hash && region.config() < config)
        });

        if let Some(region_id) = self.lookup_region_ids.get(insertion_index) {
            let region = shell
                .all_regions
                .iter()
                .find(|region| region.id() == *region_id)
                .unwrap();
            if region.config_hash() == hash && region.config() == config {
                return *region_id;
            }
        }

        let region_id = shell.all_regions.len() as u64;
        shell
            .all_regions
            .push(StagedGeneratePrintRegion::new(region_id, config));
        self.lookup_region_ids.insert(insertion_index, region_id);
        region_id
    }

    pub(super) fn region_ids_in_lookup_order(&self) -> &[u64] {
        &self.lookup_region_ids
    }
}
