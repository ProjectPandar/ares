use crate::{
    golden,
    runner::{OrcaRunner, application, stages},
};
use std::{fs, path::Path};

const IOI: &[u8] = include_bytes!("../../../../tests/parity/arachne-ioi-proximity/mk4-ioi.3mf");
const IOI_SHARED: &[u8] =
    include_bytes!("../../../../tests/parity/arachne-ioi-proximity/mk4-ioi-shared.3mf");
const IO: &[u8] = include_bytes!("../../../../tests/parity/arachne-ioi-proximity/mk4-io.3mf");
const OI: &[u8] = include_bytes!("../../../../tests/parity/arachne-ioi-proximity/mk4-oi.3mf");

#[test]
fn mk4_arachne_orders_variable_width_inner_walls_by_proximity() {
    compare_plate(IOI, "inner-outer-inner wall");
}

#[test]
fn mk4_arachne_prints_shared_inner_walls_with_their_outer_group() {
    compare_plate(IOI_SHARED, "inner-outer-inner wall");
}

#[test]
fn mk4_arachne_retains_inner_before_outer_order_on_a_slotted_plate() {
    compare_plate(IO, "inner wall/outer wall");
}

#[test]
fn mk4_arachne_retains_outer_before_inner_order_on_a_slotted_plate() {
    compare_plate(OI, "outer wall/inner wall");
}

fn compare_plate(input: &[u8], sequence: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("Arachne proximity requires Native OrcaSlicer");
        return;
    };
    let catalog = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG")
        .expect("MK4 projects require a project preset catalog");
    let catalog = Path::new(&catalog);
    let datadir = catalog.parent().unwrap().join("native");
    let dir = tempfile::Builder::new()
        .prefix("arachne-proximity-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = dir.join("input.3mf");
    let native_dir = dir.join("native");
    fs::create_dir(&native_dir).unwrap();
    fs::write(&project, input).unwrap();
    stages::run(
        runner.bin(),
        [
            "--datadir",
            datadir.to_str().unwrap(),
            "--slice",
            "1",
            "--outputdir",
            native_dir.to_str().unwrap(),
            project.to_str().unwrap(),
        ]
        .map(str::to_owned),
        stages::Stage::Slice,
        &dir,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let native_text = std::str::from_utf8(&native).unwrap();
    for setting in [
        "; wall_generator = arachne",
        &format!("; wall_sequence = {sequence}"),
    ] {
        assert!(
            native_text.lines().any(|line| line == setting),
            "{setting}: {dir:?}"
        );
    }

    let output = dir.join("ares.gcode");
    let cli = application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg("--project-preset-catalog")
        .arg(catalog)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(dir.join("ares.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "Ares returned {}: {dir:?}",
        cli.status
    );
    golden::compare_ordered_bytes_generator_only(&native, &fs::read(output).unwrap())
        .unwrap_or_else(|error| panic!("{sequence}: {error}; artifacts: {dir:?}"));
}
