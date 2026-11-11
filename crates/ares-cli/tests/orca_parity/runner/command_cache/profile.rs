use serde::Serialize;
use std::time::Instant;

#[derive(Default, Serialize)]
pub(super) struct Profile {
    pub(super) normalize_us: u128,
    pub(super) input_hash_us: u128,
    pub(super) datadir_hash_us: u128,
    pub(super) datadir_hashes: usize,
    pub(super) datadir_reused: bool,
    pub(super) environment_us: u128,
    pub(super) producer_us: u128,
    pub(super) verify_us: u128,
    pub(super) materialize_us: u128,
    pub(super) publication_us: u128,
}

pub(super) fn timed<T>(elapsed: &mut u128, run: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = run();
    *elapsed += start.elapsed().as_micros();
    result
}
