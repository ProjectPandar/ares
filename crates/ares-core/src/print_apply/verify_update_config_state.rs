use super::print_region_state::{
    StagedPrintRegionConfigKey, StagedPrintRegionRefCount, staged_print_region_ref_cnt,
    staged_print_region_ref_inc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedMissingOverrideConfigGate {
    parent_region_id: usize,
    parent_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    requires_reslice: bool,
}

impl StagedMissingOverrideConfigGate {
    pub(super) fn new(
        parent_region_id: usize,
        parent_config: StagedPrintRegionConfigKey,
        derived_config: StagedPrintRegionConfigKey,
        requires_reslice: bool,
    ) -> Self {
        Self {
            parent_region_id,
            parent_config,
            derived_config,
            requires_reslice,
        }
    }
}

pub(super) fn staged_verify_update_missing_override_config_gate(
    parent_region_id: usize,
    parent_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
) -> StagedMissingOverrideConfigGate {
    StagedMissingOverrideConfigGate::new(
        parent_region_id,
        parent_config,
        derived_config,
        derived_config != parent_config,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedExistingRegionConfigChange {
    volume_region_id: usize,
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    config_changed: bool,
}

impl StagedExistingRegionConfigChange {
    pub(super) fn new(
        volume_region_id: usize,
        current_config: StagedPrintRegionConfigKey,
        derived_config: StagedPrintRegionConfigKey,
        config_changed: bool,
    ) -> Self {
        Self {
            volume_region_id,
            current_config,
            derived_config,
            config_changed,
        }
    }

    fn config_changed(&self) -> bool {
        self.config_changed
    }
}

pub(super) fn staged_verify_update_existing_region_config_change(
    volume_region_id: usize,
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
) -> StagedExistingRegionConfigChange {
    StagedExistingRegionConfigChange::new(
        volume_region_id,
        current_config,
        derived_config,
        derived_config != current_config,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedExistingRegionUpdateAction {
    Unchanged,
    UpdateInPlace,
    RequiresReslice,
}

pub(super) fn staged_verify_update_existing_region_update_gate(
    config_change: StagedExistingRegionConfigChange,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedConfigValue {
    key: String,
    fingerprint: u64,
}

impl StagedConfigValue {
    pub(super) fn new(key: &str, fingerprint: u64) -> Self {
        Self {
            key: key.to_owned(),
            fingerprint,
        }
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn set_fingerprint(&mut self, fingerprint: u64) {
        self.fingerprint = fingerprint;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedExistingRegionConfigDiff {
    keys: Vec<String>,
}

impl StagedExistingRegionConfigDiff {
    pub(super) fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }

    fn keys(&self) -> &[String] {
        &self.keys
    }
}

pub(super) fn staged_verify_update_existing_region_config_diff(
    action: StagedExistingRegionUpdateAction,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
) -> StagedExistingRegionConfigDiff {
    if action != StagedExistingRegionUpdateAction::UpdateInPlace {
        return StagedExistingRegionConfigDiff::new(Vec::new());
    }

    let mut keys = Vec::new();
    for current_value in current_config {
        if let Some(derived_value) = derived_config
            .iter()
            .find(|derived_value| derived_value.key == current_value.key)
            && current_value.fingerprint != derived_value.fingerprint
        {
            keys.push(current_value.key.clone());
        }
    }
    StagedExistingRegionConfigDiff::new(keys)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedExistingRegionInvalidateEvent {
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    diff_keys: Vec<String>,
}

impl StagedExistingRegionInvalidateEvent {
    pub(super) fn new(
        current_config: StagedPrintRegionConfigKey,
        derived_config: StagedPrintRegionConfigKey,
        diff_keys: Vec<String>,
    ) -> Self {
        Self {
            current_config,
            derived_config,
            diff_keys,
        }
    }
}

pub(super) fn staged_verify_update_existing_region_invalidate_event(
    action: StagedExistingRegionUpdateAction,
    current_config: StagedPrintRegionConfigKey,
    derived_config: StagedPrintRegionConfigKey,
    diff: &StagedExistingRegionConfigDiff,
) -> Option<StagedExistingRegionInvalidateEvent> {
    if action == StagedExistingRegionUpdateAction::UpdateInPlace {
        Some(StagedExistingRegionInvalidateEvent::new(
            current_config,
            derived_config,
            diff.keys().to_vec(),
        ))
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedExistingRegionConfigApply {
    values: Vec<StagedConfigValue>,
    ignore_nonexistent: bool,
    hash_refreshed: bool,
}

impl StagedExistingRegionConfigApply {
    pub(super) fn new(
        values: Vec<StagedConfigValue>,
        ignore_nonexistent: bool,
        hash_refreshed: bool,
    ) -> Self {
        Self {
            values,
            ignore_nonexistent,
            hash_refreshed,
        }
    }
}

pub(super) fn staged_verify_update_existing_region_config_apply(
    event: Option<&StagedExistingRegionInvalidateEvent>,
    current_config: &[StagedConfigValue],
    derived_config: &[StagedConfigValue],
    diff: &StagedExistingRegionConfigDiff,
) -> Option<StagedExistingRegionConfigApply> {
    event?;

    let mut values = current_config.to_vec();
    for key in diff.keys() {
        if let Some(derived_value) = derived_config
            .iter()
            .find(|derived_value| derived_value.key() == key)
            && let Some(current_value) = values
                .iter_mut()
                .find(|current_value| current_value.key() == key)
        {
            current_value.set_fingerprint(derived_value.fingerprint);
        }
    }

    Some(StagedExistingRegionConfigApply::new(values, false, true))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedExistingRegionRefIncrement {
    count_after: i32,
}

impl StagedExistingRegionRefIncrement {
    pub(super) fn new(count_after: i32) -> Self {
        Self { count_after }
    }
}

pub(super) fn staged_verify_update_existing_region_ref_inc(
    action: StagedExistingRegionUpdateAction,
    apply: Option<&StagedExistingRegionConfigApply>,
    region: &mut StagedPrintRegionRefCount,
) -> Option<StagedExistingRegionRefIncrement> {
    match action {
        StagedExistingRegionUpdateAction::Unchanged => {}
        StagedExistingRegionUpdateAction::UpdateInPlace => {
            apply?;
        }
        StagedExistingRegionUpdateAction::RequiresReslice => return None,
    }

    staged_print_region_ref_inc(region);
    Some(StagedExistingRegionRefIncrement::new(
        staged_print_region_ref_cnt(region),
    ))
}
