//! Multi-project slice-API isolation: the adaptive octree cache is bound
//! to the slice context (`PreparedPostClassicTraversal::adaptive_octrees`),
//! so interleaved `slice_project` calls with different projects can never
//! observe each other's trees. This is the regression test for that
//! guarantee — two adaptive projects at different densities, sliced
//! twice each in one process, must produce byte-identical outputs per
//! project (fixed `GenerationMetadata` makes a slice fully deterministic).

use ares_core::{GenerationMetadata, slice_project};
use std::{fs, path::Path};

async fn slice_fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/adaptive")
        .join(name);
    let project = fs::read(path).unwrap();
    let metadata = GenerationMetadata::new_local(2026, 9, 15, 0, 0, 0).unwrap();
    slice_project(project, metadata).await.expect("slice")
}

#[tokio::test]
async fn interleaved_adaptive_projects_are_isolated() {
    let d15_first = slice_fixture("d15.3mf").await;
    let d30_first = slice_fixture("d30.3mf").await;
    let d15_second = slice_fixture("d15.3mf").await;
    let d30_second = slice_fixture("d30.3mf").await;

    assert_eq!(
        d15_first, d15_second,
        "second d15 slice changed — cross-project cache leak"
    );
    assert_eq!(
        d30_first, d30_second,
        "second d30 slice changed — cross-project cache leak"
    );
    assert_ne!(
        d15_first, d30_first,
        "the two density variants must produce different G-code"
    );
}
