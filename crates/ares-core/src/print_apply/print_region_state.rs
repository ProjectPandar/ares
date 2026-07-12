#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct StagedPrintRegionRefCount {
    ref_count: i32,
}

pub(super) fn staged_print_region_ref_inc(region: &mut StagedPrintRegionRefCount) {
    region.ref_count += 1;
}

pub(super) fn staged_print_region_ref_reset(region: &mut StagedPrintRegionRefCount) {
    region.ref_count = 0;
}

pub(super) fn staged_print_region_ref_cnt(region: &StagedPrintRegionRefCount) -> i32 {
    region.ref_count
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagedPrintRegionConfigKey {
    fingerprint: u64,
}

impl StagedPrintRegionConfigKey {
    pub(super) fn new(fingerprint: u64) -> Self {
        Self { fingerprint }
    }
}
