use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum StagedModelObjectApplyStatus {
    Unknown,
    Old,
    New,
    Moved,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum StagedPrintObjectRegionsStatus {
    Invalid,
    Valid,
    PartiallyValid,
}

#[derive(Clone, Debug)]
pub(super) struct StagedModelObjectStatus {
    pub(super) id: u64,
    pub(super) status: StagedModelObjectApplyStatus,
    pub(super) print_object_regions_status: StagedPrintObjectRegionsStatus,
}

impl StagedModelObjectStatus {
    pub(super) fn new(id: u64) -> Self {
        Self::with_status(id, StagedModelObjectApplyStatus::Unknown)
    }

    pub(super) fn with_status(id: u64, status: StagedModelObjectApplyStatus) -> Self {
        Self {
            id,
            status,
            print_object_regions_status: StagedPrintObjectRegionsStatus::Invalid,
        }
    }
}

impl PartialEq for StagedModelObjectStatus {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StagedModelObjectStatus {}

impl PartialOrd for StagedModelObjectStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StagedModelObjectStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Default)]
pub(super) struct StagedModelObjectStatusDb {
    records: BTreeMap<u64, StagedModelObjectStatus>,
}

impl StagedModelObjectStatusDb {
    pub(super) fn add(&mut self, id: u64, status: StagedModelObjectApplyStatus) {
        use std::collections::btree_map::Entry;

        match self.records.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(StagedModelObjectStatus::with_status(id, status));
            }
            Entry::Occupied(_) => panic!("duplicate model object status id"),
        }
    }

    pub(super) fn add_if_new(&mut self, id: u64, status: StagedModelObjectApplyStatus) -> bool {
        use std::collections::btree_map::Entry;

        match self.records.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(StagedModelObjectStatus::with_status(id, status));
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    pub(super) fn get(&self, id: u64) -> &StagedModelObjectStatus {
        self.records.get(&id).unwrap()
    }

    pub(super) fn reuse(&self, id: u64) -> &StagedModelObjectStatus {
        let status = self.get(id);
        assert_ne!(status.status, StagedModelObjectApplyStatus::Deleted);
        status
    }

    pub(super) fn records(&self) -> impl Iterator<Item = &StagedModelObjectStatus> {
        self.records.values()
    }
}
