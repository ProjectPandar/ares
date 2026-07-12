#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArchiveLimits {
    pub max_entries: usize,
    pub max_entry_size: u64,
    pub max_total_size: u64,
    pub max_expansion_ratio: u64,
}

impl ArchiveLimits {
    pub(crate) const PROJECT: Self = Self {
        max_entries: 4_096,
        max_entry_size: 256 * 1024 * 1024,
        max_total_size: 1024 * 1024 * 1024,
        max_expansion_ratio: 1_000,
    };
}
