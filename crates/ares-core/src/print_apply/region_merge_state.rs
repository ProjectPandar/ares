#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedRegionMergeRegion {
    region_id: usize,
    config_fingerprint: u64,
    config_hash: u64,
    ref_count: i32,
}

impl StagedRegionMergeRegion {
    pub(super) fn new(
        region_id: usize,
        config_fingerprint: u64,
        config_hash: u64,
        ref_count: i32,
    ) -> Self {
        Self {
            region_id,
            config_fingerprint,
            config_hash,
            ref_count,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StagedMergeVerificationResult {
    Valid,
    RequiresReslice {
        first_region_id: usize,
        second_region_id: usize,
    },
}

pub(super) fn staged_region_merge_verification(
    regions: &[StagedRegionMergeRegion],
) -> StagedMergeVerificationResult {
    for region in regions {
        assert!(
            region.ref_count > 0,
            "print region ref count must be positive"
        );
    }

    let mut regions = regions.to_vec();
    regions.sort_by_key(|region| region.config_hash);

    let mut i = 0;
    while i < regions.len() {
        let hash = regions[i].config_hash;
        let mut j = i + 1;
        while j < regions.len() && regions[j].config_hash == hash {
            if regions[i].config_fingerprint == regions[j].config_fingerprint {
                return StagedMergeVerificationResult::RequiresReslice {
                    first_region_id: regions[i].region_id,
                    second_region_id: regions[j].region_id,
                };
            }
            j += 1;
        }
        i += 1;
    }

    StagedMergeVerificationResult::Valid
}
