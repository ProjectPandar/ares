//! `OptionOutcome` bookkeeping for option-domain coverage legs — moved out
//! of `option_coverage.rs` to keep the index file under the 400-LOC cap.
//! Re-exported at the module root so `super::{OptionOutcome, …}` paths in
//! existing coverage legs stay unchanged.

use crate::runner;
use std::path::PathBuf;

pub(super) fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

pub(super) fn strict_comparator_ran(status: &str) -> bool {
    matches!(status, "PASS" | "DIVERGENT")
}
