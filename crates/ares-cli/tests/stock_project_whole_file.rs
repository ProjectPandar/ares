#[path = "stock_project_whole_file/golden.rs"]
mod golden;
// Diagnostic-only parsers and behavioral tests, never acceptance fallback.
#[path = "stock_project_whole_file/semantic.rs"]
mod semantic;
#[path = "stock_project_whole_file/tests.rs"]
mod tests;

use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};

const PROJECT_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ksr_fdmtest_v4")
        .join(name)
}

fn project() -> Vec<u8> {
    fs::read(fixture_path("ksr_fdmtest_v4.project.3mf")).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
