use super::{OrcaRunner, golden, native};
use crate::runner::application::CliCommand;
use std::fs;

#[test]
fn emits_the_native_untwisted_polyhole_perimeter() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        include_bytes!("../../../../../../tests/parity/hole-to-polyhole/untwisted-fine-radius.3mf");
    let root = tempfile::Builder::new()
        .prefix("untwisted-polyhole-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let reference = native(&runner, &directory, project);
    assert!(
        std::str::from_utf8(&reference)
            .unwrap()
            .lines()
            .any(|line| line == "G1 X132.838 Y117.729 E.0491"),
        "Native did not emit the expected untwisted perimeter: {root:?}"
    );

    let output = directory.join("ares.gcode");
    let cli = CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI exit {:?}: {root:?}", cli.status);
    golden::compare_ordered_bytes_generator_only(&reference, &fs::read(output).unwrap())
        .unwrap_or_else(|error| panic!("{root:?}: {error}"));
}
