use super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::model_volume_state::{StagedModelVolumeType, staged_model_volume_solid_or_modifier};
use super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents, staged_find_volume_extents};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGenerateModelPartVolume {
    id: u64,
    volume_type: StagedModelVolumeType,
    config: StagedGenerateRegionConfigKey,
}

impl StagedGenerateModelPartVolume {
    pub(super) fn new(
        id: u64,
        volume_type: StagedModelVolumeType,
        config: StagedGenerateRegionConfigKey,
    ) -> Self {
        Self {
            id,
            volume_type,
            config,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedGenerateModelPartLayer {
    volumes: Vec<StagedVolumeExtents>,
    volume_regions: Vec<StagedGenerateModelPartVolumeRegion>,
}

impl StagedGenerateModelPartLayer {
    pub(super) fn new(volumes: Vec<StagedVolumeExtents>) -> Self {
        Self {
            volumes,
            volume_regions: Vec::new(),
        }
    }

    pub(super) fn volume_regions(&self) -> &[StagedGenerateModelPartVolumeRegion] {
        &self.volume_regions
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedGenerateModelPartVolumeRegion {
    volume_id: u64,
    parent: isize,
    region_id: u64,
    bbox: StagedExtentBox,
}

impl StagedGenerateModelPartVolumeRegion {
    pub(super) fn volume_id(&self) -> u64 {
        self.volume_id
    }

    pub(super) fn parent(&self) -> isize {
        self.parent
    }

    pub(super) fn region_id(&self) -> u64 {
        self.region_id
    }

    pub(super) fn bbox(&self) -> StagedExtentBox {
        self.bbox
    }
}

pub(super) fn staged_generate_model_part_volume_regions(
    shell: &mut StagedGeneratePrintObjectRegions,
    layers: &mut [StagedGenerateModelPartLayer],
    model_volumes: &[StagedGenerateModelPartVolume],
    region_set: &mut StagedGenerateRegionSet,
) {
    for volume in model_volumes {
        if !staged_model_volume_solid_or_modifier(volume.volume_type) {
            continue;
        }
        if volume.volume_type != StagedModelVolumeType::ModelPart {
            continue;
        }

        for layer in layers.iter_mut() {
            let Some(bbox) = staged_find_volume_extents(&layer.volumes, volume.id) else {
                continue;
            };
            let region_id = region_set.get_create_region(shell, volume.config);
            layer
                .volume_regions
                .push(StagedGenerateModelPartVolumeRegion {
                    volume_id: volume.id,
                    parent: -1,
                    region_id,
                    bbox: *bbox,
                });
        }
    }
}
