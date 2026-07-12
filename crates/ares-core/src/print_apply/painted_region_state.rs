use super::print_region_state::StagedPrintRegionConfigKey;
use super::print_region_state::{StagedPrintRegionRefCount, staged_print_region_ref_cnt};
use super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigDiff,
    StagedExistingRegionInvalidateEvent, StagedExistingRegionRefIncrement,
    StagedExistingRegionUpdateAction, staged_verify_update_existing_region_config_apply,
    staged_verify_update_existing_region_config_diff,
    staged_verify_update_existing_region_invalidate_event,
    staged_verify_update_existing_region_ref_inc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedPaintedRegionConfig {
    marker: u64,
    wall_filament: u32,
    solid_infill_filament: u32,
    sparse_infill_filament: u32,
}

impl StagedPaintedRegionConfig {
    pub(super) fn new(
        marker: u64,
        wall_filament: u32,
        solid_infill_filament: u32,
        sparse_infill_filament: u32,
    ) -> Self {
        Self {
            marker,
            wall_filament,
            solid_infill_filament,
            sparse_infill_filament,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedPaintedRegionInput {
    painted_region_id: usize,
    parent_volume_region_id: usize,
    extruder_id: u32,
}

impl StagedPaintedRegionInput {
    pub(super) fn new(
        painted_region_id: usize,
        parent_volume_region_id: usize,
        extruder_id: u32,
    ) -> Self {
        Self {
            painted_region_id,
            parent_volume_region_id,
            extruder_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedPaintedRegionConfigDerivation {
    painted_region_id: usize,
    parent_volume_region_id: usize,
    config: StagedPaintedRegionConfig,
}

impl StagedPaintedRegionConfigDerivation {
    pub(super) fn new(
        painted_region_id: usize,
        parent_volume_region_id: usize,
        config: StagedPaintedRegionConfig,
    ) -> Self {
        Self {
            painted_region_id,
            parent_volume_region_id,
            config,
        }
    }
}

pub(super) fn staged_painted_region_extruder_configs(
    parent_configs: &[StagedPaintedRegionConfig],
    painted_regions: &[StagedPaintedRegionInput],
) -> Vec<StagedPaintedRegionConfigDerivation> {
    painted_regions
        .iter()
        .map(|painted_region| {
            let mut config = parent_configs[painted_region.parent_volume_region_id];
            config.wall_filament = painted_region.extruder_id;
            config.solid_infill_filament = painted_region.extruder_id;
            config.sparse_infill_filament = painted_region.extruder_id;
            StagedPaintedRegionConfigDerivation::new(
                painted_region.painted_region_id,
                painted_region.parent_volume_region_id,
                config,
            )
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedPaintedRegionConfigChange {
    painted_region_id: usize,
    current_config: StagedPaintedRegionConfig,
    derived_config: StagedPaintedRegionConfig,
    config_changed: bool,
}

impl StagedPaintedRegionConfigChange {
    pub(super) fn new(
        painted_region_id: usize,
        current_config: StagedPaintedRegionConfig,
        derived_config: StagedPaintedRegionConfig,
        config_changed: bool,
    ) -> Self {
        Self {
            painted_region_id,
            current_config,
            derived_config,
            config_changed,
        }
    }

    fn config_changed(&self) -> bool {
        self.config_changed
    }
}

pub(super) fn staged_painted_region_config_change(
    painted_region_id: usize,
    current_config: StagedPaintedRegionConfig,
    derived_config: StagedPaintedRegionConfig,
) -> StagedPaintedRegionConfigChange {
    StagedPaintedRegionConfigChange::new(
        painted_region_id,
        current_config,
        derived_config,
        derived_config != current_config,
    )
}

pub(super) fn staged_painted_region_update_gate(
    config_change: StagedPaintedRegionConfigChange,
    region: &StagedPrintRegionRefCount,
) -> StagedExistingRegionUpdateAction {
    if !config_change.config_changed() {
        StagedExistingRegionUpdateAction::Unchanged
    } else if staged_print_region_ref_cnt(region) == 0 {
        StagedExistingRegionUpdateAction::UpdateInPlace
    } else {
        StagedExistingRegionUpdateAction::RequiresReslice
    }
}

pub(super) fn staged_painted_region_config_diff(
    action: StagedExistingRegionUpdateAction,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
) -> StagedExistingRegionConfigDiff {
    staged_verify_update_existing_region_config_diff(action, current_config, derived_config)
}

pub(super) fn staged_painted_region_invalidate_event(
    action: StagedExistingRegionUpdateAction,
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    diff: &StagedExistingRegionConfigDiff,
) -> Option<StagedExistingRegionInvalidateEvent> {
    staged_verify_update_existing_region_invalidate_event(
        action,
        current_config,
        derived_config,
        diff,
    )
}

pub(super) fn staged_painted_region_config_apply(
    event: Option<&StagedExistingRegionInvalidateEvent>,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
    diff: &StagedExistingRegionConfigDiff,
) -> Option<StagedExistingRegionConfigApply> {
    staged_verify_update_existing_region_config_apply(event, current_config, derived_config, diff)
}

pub(super) fn staged_painted_region_ref_inc(
    action: StagedExistingRegionUpdateAction,
    apply: Option<&StagedExistingRegionConfigApply>,
    region: &mut StagedPrintRegionRefCount,
) -> Option<StagedExistingRegionRefIncrement> {
    staged_verify_update_existing_region_ref_inc(action, apply, region)
}
