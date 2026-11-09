//! Inputs and recorded outputs shared by the parity runner stages.

#[path = "post_region_normalization.rs"]
mod post_region_normalization;

use serde_json::{Map, Value};
use std::path::PathBuf;

pub(crate) struct ExportedCase {
    pub(crate) label: String,
    pub(crate) project: Vec<u8>,
    pub(super) project_path: PathBuf,
    pub(super) output_dir: PathBuf,
}

pub(crate) struct ParityCase {
    /// Human-readable case label (vendor / printer / variant).
    pub(crate) label: String,
    pub(crate) project: Vec<u8>,
    pub(crate) reference: Vec<u8>,
}

/// Flattened preset inputs for one parity case.
pub(crate) struct CaseInputs<'a> {
    pub(crate) label: &'a str,
    pub(crate) machine: &'a Map<String, Value>,
    pub(crate) process: &'a Map<String, Value>,
    pub(crate) filaments: &'a [Map<String, Value>],
}
