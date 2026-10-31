//! `SeamPlacer.cpp:44–48,784–785,930–941`: nearest seam distance Gaussian.

use super::{ExportedCase, OrcaRunner};
use sha2::{Digest, Sha256};

#[test]
fn dumbbell_nearest_seam_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        include_bytes!("../../../../../tests/parity/validation/tiq2-dumbbell-long-loop.3mf");
    assert_eq!(
        Sha256::digest(project)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "90288cb1f8904fc245a792be9cc8d30e5ea2762f6faff4b573d362fec231bc23"
    );
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, project).unwrap();
    let output_dir = runner.work.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&ExportedCase {
            label: "nearest-seam/dumbbell-gaussian".into(),
            project: project.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; artifacts: {:?}",
        outcome.detail, outcome.artifacts
    );
}
