use super::{OrcaRunner, golden, native};
use crate::runner::application::CliCommand;
use std::fs;

#[test]
fn emits_the_native_twisted_polyhole_vertices() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        include_bytes!("../../../../../../tests/parity/hole-to-polyhole/twisted-43-sincos.3mf");
    let root = tempfile::Builder::new()
        .prefix("twisted-polyhole-trig-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let reference = native(&runner, &directory, project);
    let reference_text = std::str::from_utf8(&reference).unwrap();
    for expected in ["G1 X132.031 Y133.447 E.0493", "G1 X134.352 Y119.2 E.05329"] {
        assert!(
            reference_text.lines().any(|line| line == expected),
            "Native did not emit expected polyhole vertex {expected}: {root:?}"
        );
    }

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
