//! Ares-defined rejection, not a Native comparison: see
//! `docs/architecture/ironing-pattern-coverage-spec.md`.

use super::{project, sha};
use serde_json::json;
use std::fs;

#[test]
fn ironing_pattern_3dhoneycomb_is_rejected_as_unsupported() {
    let project = project("3dhoneycomb");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let error = runtime
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .expect_err("3D honeycomb ironing has no defined result");
    let error = format!("{error:?}");
    let artifacts = tempfile::Builder::new()
        .prefix("ironing-pattern-3dhoneycomb-guard-")
        .tempdir_in(
            std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
                .map_or_else(std::env::temp_dir, std::path::PathBuf::from),
        )
        .unwrap()
        .keep();
    fs::write(
        artifacts.join("guard.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": sha(&project),
            "native_comparison": false,
            "ares_error": error,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(error, r#"UnsupportedProjectFeature("ironing_pattern")"#);
}
