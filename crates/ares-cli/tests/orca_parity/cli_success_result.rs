//! The CLI's success report (`record_exit_reson(outfile_dir, 0, plate_to_slice, …)`,
//! `OrcaSlicer.cpp:8126-8136` at v2.5.0-beta `345efec64`; see
//! `docs/architecture/beta-cli-success-result.md`): a one-plate 3MF slice exits 0 with silent
//! stdout/stderr and writes result.json with "Success.", the plate's settings values and one
//! `sliced_plates` entry whose `triangle_count` sums the model-part facets of each inside instance.

use std::{fs, path::Path};

use crate::native_wall_clock::result_comparison_bytes;
use crate::runner::{OrcaRunner, stages};

#[path = "support_necessity_warning.rs"]
mod support_necessity_warning;

/// One 10 mm cube on an Ender-3: 12 facets.
#[test]
fn cli_success_result_matches_native_for_one_cube() {
    assert_success_like_native("cube", "tests/parity/orca_cli_ender3.3mf");
}

/// Two cube objects with three instances on one plate: the count is per instance (36 facets).
#[test]
fn cli_success_result_counts_triangles_per_instance_like_native() {
    assert_success_like_native(
        "three-cubes",
        "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf",
    );
}

fn assert_success_like_native(label: &str, fixture: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-success-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = artifacts.join("input.3mf");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(fixture),
        &project,
    )
    .unwrap();
    let native = artifacts.join("native");
    let ares = artifacts.join("ares");
    fs::create_dir(&native).unwrap();
    fs::create_dir(&ares).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            native.to_str().unwrap().to_owned(),
            project.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &artifacts,
    )
    .unwrap();
    let command = fs::read_dir(&artifacts)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("command-")
        })
        .unwrap();
    let output = crate::runner::application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(ares.join("plate_1.gcode"))
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{label}: {artifacts:?}");
    assert_eq!(
        output.stdout,
        fs::read(command.join("stdout.bin")).unwrap(),
        "{label}: stdout"
    );
    assert_eq!(
        output.stderr,
        fs::read(command.join("stderr.bin")).unwrap(),
        "{label}: stderr"
    );
    assert_eq!(
        result_comparison_bytes(&fs::read(ares.join("result.json")).unwrap()),
        result_comparison_bytes(&fs::read(native.join("result.json")).unwrap()),
        "{label}: complete result.json bytes: {artifacts:?}"
    );
}
