//! `GCode.cpp:3481-3491`: print-end template dispatch.
use super::{ExportedCase, OrcaRunner, cache_identity};

#[test]
fn tiq2_footer_dispatch_whole_file() {
    let input = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(input),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("TiQ2 whole-file comparison: no optional OrcaRunner");
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("tiq2-footer-dispatch-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = crate::print_validation::tiq2_corner::corner_input(
        input,
        crate::print_validation::tiq2_corner::Scenario::WholeFile,
        &directory,
    );
    let path = directory.join("input.3mf");
    std::fs::write(&path, &input).unwrap();
    let output_dir = directory.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "TiQ2/footer-dispatch/whole-file".to_owned(),
            project: input.to_vec(),
            project_path: path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
