//! Actual WASM/browser bytes against Print::export_gcode / GCode::do_export.
//! The capture comes from web/e2e/ksrProject.spec.ts; no CLI substitution.

use super::{compare, producer};
use crate::runner::{ExportedCase, cache_identity, repo_root};
use std::fs;

#[test]
fn browser_output_matches_initialized_orca_ordered_bytes() {
    let Some(capture) = std::env::var_os("ARES_BROWSER_GCODE") else {
        crate::native_environment::skipped(
            "browser capture absent; run via web/e2e/ksrProject.spec.ts",
        );
        return;
    };
    let Some(runner) = producer::runner(
        "runner::stock_project_native_stream::browser::browser_output_matches_initialized_orca_ordered_bytes",
    ) else {
        return;
    };
    let actual = fs::read(capture).unwrap();
    let source = repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
    let project = fs::read(&source).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&project),
        "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
        "{:?}",
        runner.work,
    );
    let project_path = runner.work.join(source.file_name().unwrap());
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "ksr/browser".to_owned(),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    fs::write(runner.work.join("orca.gcode"), &case.reference).unwrap();
    fs::write(runner.work.join("browser.gcode"), &actual).unwrap();
    compare(&case.reference, &actual, &runner.work).unwrap();
}
