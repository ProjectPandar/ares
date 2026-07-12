use super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::model_volume_state::StagedModelVolumeType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePaintedParentVolumeRegion {
    volume_type: StagedModelVolumeType,
    parent_config_marker: u64,
}

impl StagedGeneratePaintedParentVolumeRegion {
    pub(super) fn new(volume_type: StagedModelVolumeType, parent_config_marker: u64) -> Self {
        Self {
            volume_type,
            parent_config_marker,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePaintedConfig {
    marker: u64,
    wall_filament: u32,
    solid_infill_filament: u32,
    sparse_infill_filament: u32,
}

impl StagedGeneratePaintedConfig {
    fn from_parent(parent_marker: u64, painted_extruder_id: u32) -> Self {
        Self {
            marker: parent_marker,
            wall_filament: painted_extruder_id,
            solid_infill_filament: painted_extruder_id,
            sparse_infill_filament: painted_extruder_id,
        }
    }

    pub(super) fn marker(&self) -> u64 {
        self.marker
    }

    pub(super) fn filaments(&self) -> (u32, u32, u32) {
        (
            self.wall_filament,
            self.solid_infill_filament,
            self.sparse_infill_filament,
        )
    }

    fn region_key(&self) -> StagedGenerateRegionConfigKey {
        let filament = u64::from(self.wall_filament);
        StagedGenerateRegionConfigKey::new(self.marker, filament)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedGeneratePaintedRegion {
    extruder_id: u32,
    parent: usize,
    region_id: u64,
    derived_config: StagedGeneratePaintedConfig,
}

impl StagedGeneratePaintedRegion {
    pub(super) fn extruder_id(&self) -> u32 {
        self.extruder_id
    }

    pub(super) fn parent(&self) -> usize {
        self.parent
    }

    pub(super) fn region_id(&self) -> u64 {
        self.region_id
    }

    pub(super) fn derived_config(&self) -> StagedGeneratePaintedConfig {
        self.derived_config
    }
}

pub(super) fn staged_generate_painted_regions(
    shell: &mut StagedGeneratePrintObjectRegions,
    parent_regions: &[StagedGeneratePaintedParentVolumeRegion],
    painting_extruders: &[u32],
    region_set: &mut StagedGenerateRegionSet,
) -> Vec<StagedGeneratePaintedRegion> {
    let mut painted_regions = Vec::new();

    for &extruder_id in painting_extruders {
        for (parent_id, parent_region) in parent_regions.iter().enumerate() {
            if parent_region.volume_type != StagedModelVolumeType::ModelPart
                && parent_region.volume_type != StagedModelVolumeType::ParameterModifier
            {
                continue;
            }

            let derived_config = StagedGeneratePaintedConfig::from_parent(
                parent_region.parent_config_marker,
                extruder_id,
            );
            let region_id = region_set.get_create_region(shell, derived_config.region_key());
            painted_regions.push(StagedGeneratePaintedRegion {
                extruder_id,
                parent: parent_id,
                region_id,
                derived_config,
            });
        }
    }

    painted_regions
}
