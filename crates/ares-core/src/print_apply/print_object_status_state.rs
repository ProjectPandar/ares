use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum StagedPrintObjectApplyStatus {
    Unknown,
    Deleted,
    Reused,
    New,
}

#[derive(Clone, Debug)]
pub(super) struct StagedPrintObjectStatus {
    pub(super) id: u64,
    pub(super) status: StagedPrintObjectApplyStatus,
}

impl StagedPrintObjectStatus {
    pub(super) fn new(id: u64) -> Self {
        Self::with_status(id, StagedPrintObjectApplyStatus::Unknown)
    }

    pub(super) fn with_status(id: u64, status: StagedPrintObjectApplyStatus) -> Self {
        Self { id, status }
    }
}

impl PartialEq for StagedPrintObjectStatus {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StagedPrintObjectStatus {}

impl PartialOrd for StagedPrintObjectStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StagedPrintObjectStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct StagedPrintObjectStatusDb {
    records: Vec<StagedPrintObjectStatus>,
}

impl StagedPrintObjectStatusDb {
    pub(super) fn from_ids(ids: impl IntoIterator<Item = u64>) -> Self {
        let mut records: Vec<_> = ids.into_iter().map(StagedPrintObjectStatus::new).collect();
        records.sort();
        Self { records }
    }

    pub(super) fn records(&self) -> impl Iterator<Item = &StagedPrintObjectStatus> {
        self.records.iter()
    }

    pub(super) fn get_range(&self, id: u64) -> impl Iterator<Item = &StagedPrintObjectStatus> {
        let start = self.records.partition_point(|record| record.id < id);
        let end = self.records.partition_point(|record| record.id <= id);
        self.records[start..end].iter()
    }

    pub(super) fn count(&self, id: u64) -> usize {
        self.get_range(id).count()
    }

    pub(super) fn clear(&mut self) {
        self.records.clear();
    }
}
