use std::collections::BTreeSet;

const PROFILE_ID_KEYS: [&str; 3] = [
    "print_settings_id",
    "filament_settings_id",
    "printer_settings_id",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedApplyNormalizationCall {
    NormalizeFdm1,
    NormalizeFdm2 {
        object_count: usize,
        used_filament_count: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedApplyNormalizationPrelude {
    materialized_profile_id_keys: Vec<&'static str>,
    used_filaments: Vec<u32>,
    used_filament_set: BTreeSet<u32>,
    calls: Vec<StagedApplyNormalizationCall>,
    changed_keys: Vec<String>,
}

impl StagedApplyNormalizationPrelude {
    pub(super) fn materialized_profile_id_keys(&self) -> &[&'static str] {
        &self.materialized_profile_id_keys
    }

    pub(super) fn used_filaments(&self) -> &[u32] {
        &self.used_filaments
    }

    pub(super) fn used_filament_set(&self) -> &BTreeSet<u32> {
        &self.used_filament_set
    }

    pub(super) fn calls(&self) -> &[StagedApplyNormalizationCall] {
        &self.calls
    }

    pub(super) fn changed_keys(&self) -> &[String] {
        &self.changed_keys
    }
}

pub(super) fn staged_apply_normalization_prelude(
    object_count: usize,
    used_filaments: &[u32],
    normalize_fdm_2_changed_keys: &[String],
) -> StagedApplyNormalizationPrelude {
    StagedApplyNormalizationPrelude {
        materialized_profile_id_keys: PROFILE_ID_KEYS.to_vec(),
        used_filaments: used_filaments.to_vec(),
        used_filament_set: used_filaments.iter().cloned().collect(),
        calls: vec![
            StagedApplyNormalizationCall::NormalizeFdm1,
            StagedApplyNormalizationCall::NormalizeFdm2 {
                object_count,
                used_filament_count: used_filaments.len(),
            },
        ],
        changed_keys: normalize_fdm_2_changed_keys.to_vec(),
    }
}
