//! `CoolingBuffer.cpp:890-898,1000-1002`: restore retained role fan speed.
use super::{ExportedCase, OrcaRunner, cache_identity};

#[test]
fn orca_parity_tiq2_retained_fan_whole_file() {
    let input = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(input),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("tiq2-retained-fan-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    std::fs::write(&path, input).unwrap();
    let output_dir = directory.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "TiQ2/retained-fan/whole-file".to_owned(),
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
