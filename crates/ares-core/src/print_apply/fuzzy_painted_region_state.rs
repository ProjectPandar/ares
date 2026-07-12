use super::print_region_state::{
    StagedPrintRegionConfigKey, StagedPrintRegionRefCount, staged_print_region_ref_cnt,
};
use super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionConfigDiff,
    StagedExistingRegionInvalidateEvent, StagedExistingRegionRefIncrement,
    StagedExistingRegionUpdateAction, staged_verify_update_existing_region_config_apply,
    staged_verify_update_existing_region_config_diff,
    staged_verify_update_existing_region_invalidate_event,
    staged_verify_update_existing_region_ref_inc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedFuzzySkinType {
    None,
    External,
    Hole,
    All,
    AllWalls,
    DisabledFuzzy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedFuzzySkinConfig {
    region_id: usize,
    marker: u64,
    fuzzy_skin: StagedFuzzySkinType,
}

impl StagedFuzzySkinConfig {
    pub(super) fn new(region_id: usize, marker: u64, fuzzy_skin: StagedFuzzySkinType) -> Self {
        Self {
            region_id,
            marker,
            fuzzy_skin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedFuzzyPaintedRegionParent {
    VolumeRegion(usize),
    PaintedRegion(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedFuzzyPaintedRegionInput {
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
}

impl StagedFuzzyPaintedRegionInput {
    pub(super) fn new(
        fuzzy_region_id: usize,
        parent: StagedFuzzyPaintedRegionParent,
        destination_region_id: usize,
    ) -> Self {
        Self {
            fuzzy_region_id,
            parent,
            destination_region_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedFuzzyPaintedRegionConfigDerivation {
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
    config: StagedFuzzySkinConfig,
}

impl StagedFuzzyPaintedRegionConfigDerivation {
    pub(super) fn new(
        fuzzy_region_id: usize,
        parent: StagedFuzzyPaintedRegionParent,
        destination_region_id: usize,
        config: StagedFuzzySkinConfig,
    ) -> Self {
        Self {
            fuzzy_region_id,
            parent,
            destination_region_id,
            config,
        }
    }
}

pub(super) fn staged_fuzzy_painted_region_configs(
    volume_parent_configs: &[StagedFuzzySkinConfig],
    painted_parent_configs: &[StagedFuzzySkinConfig],
    fuzzy_regions: &[StagedFuzzyPaintedRegionInput],
) -> Vec<StagedFuzzyPaintedRegionConfigDerivation> {
    fuzzy_regions
        .iter()
        .map(|fuzzy_region| {
            let mut config = match fuzzy_region.parent {
                StagedFuzzyPaintedRegionParent::VolumeRegion(parent) => {
                    volume_parent_configs[parent]
                }
                StagedFuzzyPaintedRegionParent::PaintedRegion(parent) => {
                    painted_parent_configs[parent]
                }
            };
            if config.fuzzy_skin != StagedFuzzySkinType::DisabledFuzzy {
                config.fuzzy_skin = StagedFuzzySkinType::All;
            }
            StagedFuzzyPaintedRegionConfigDerivation::new(
                fuzzy_region.fuzzy_region_id,
                fuzzy_region.parent,
                fuzzy_region.destination_region_id,
                config,
            )
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedFuzzyPaintedRegionConfigChange {
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
    current_config: StagedFuzzySkinConfig,
    derived_config: StagedFuzzySkinConfig,
    config_changed: bool,
}

impl StagedFuzzyPaintedRegionConfigChange {
    pub(super) fn new(
        derivation: StagedFuzzyPaintedRegionConfigDerivation,
        current_config: StagedFuzzySkinConfig,
        config_changed: bool,
    ) -> Self {
        Self {
            fuzzy_region_id: derivation.fuzzy_region_id,
            parent: derivation.parent,
            destination_region_id: derivation.destination_region_id,
            current_config,
            derived_config: derivation.config,
            config_changed,
        }
    }

    fn config_changed(&self) -> bool {
        self.config_changed
    }
}

pub(super) fn staged_fuzzy_painted_region_config_change(
    derivation: StagedFuzzyPaintedRegionConfigDerivation,
    current_config: StagedFuzzySkinConfig,
) -> StagedFuzzyPaintedRegionConfigChange {
    StagedFuzzyPaintedRegionConfigChange::new(
        derivation,
        current_config,
        derivation.config != current_config,
    )
}

pub(super) fn staged_fuzzy_painted_region_update_gate(
    config_change: StagedFuzzyPaintedRegionConfigChange,
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

pub(super) fn staged_fuzzy_painted_region_config_diff(
    action: StagedExistingRegionUpdateAction,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
) -> StagedExistingRegionConfigDiff {
    staged_verify_update_existing_region_config_diff(action, current_config, derived_config)
}

pub(super) fn staged_fuzzy_painted_region_invalidate_event(
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

pub(super) fn staged_fuzzy_painted_region_config_apply(
    event: Option<&StagedExistingRegionInvalidateEvent>,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
    diff: &StagedExistingRegionConfigDiff,
) -> Option<StagedExistingRegionConfigApply> {
    staged_verify_update_existing_region_config_apply(event, current_config, derived_config, diff)
}

pub(super) fn staged_fuzzy_painted_region_ref_inc(
    action: StagedExistingRegionUpdateAction,
    apply: Option<&StagedExistingRegionConfigApply>,
    region: &mut StagedPrintRegionRefCount,
) -> Option<StagedExistingRegionRefIncrement> {
    staged_verify_update_existing_region_ref_inc(action, apply, region)
}
