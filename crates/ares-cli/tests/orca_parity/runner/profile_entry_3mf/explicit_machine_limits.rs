use std::{fs, path::Path};

use serde_json::{Value, json};

use crate::{
    golden,
    runner::{OrcaRunner, application::cli_command, profile_entry::slice_flattened, stages},
};

const FIXTURES: &str = "tests/parity/cli-explicit-machine-limits";
const X: &str = "machine_max_acceleration_x";
const Y: &str = "machine_max_acceleration_y";
const TRAVEL: &str = "machine_max_acceleration_travel";

#[test]
fn explicit_machine_limits_clamp_stock_stl() {
    compare("cube10.stl", |_| {}, ["6000,6000"; 3]);
}

#[test]
fn explicit_machine_limits_clamp_x_only_stl() {
    compare("cube10.stl", preclamp_y_and_travel, ["6000,6000"; 3]);
}

#[test]
fn explicit_machine_limits_keep_preclamped_stl() {
    compare(
        "cube10.stl",
        |machine| {
            preclamp_y_and_travel(machine);
            machine[X] = json!(["6000", "6000"]);
        },
        ["6000,6000"; 3],
    );
}

#[test]
fn explicit_machine_limits_cap_individual_slots_without_raising_or_padding() {
    compare(
        "cube10.stl",
        |machine| {
            machine[X] = json!(["5000", "20000", "16000"]);
            machine[Y] = json!(["4000"]);
        },
        ["5000,6000,16000", "4000", "6000,6000"],
    );
}

#[test]
fn explicit_machine_limits_keep_same_name_3mf_uncapped() {
    compare(
        "a1mini.3mf",
        |_| {},
        ["20000,20000", "20000,20000", "9000,9000"],
    );
}

#[test]
fn explicit_machine_limits_clamp_renamed_3mf() {
    compare("renamed.3mf", |_| {}, ["6000,6000"; 3]);
}

fn preclamp_y_and_travel(machine: &mut Value) {
    machine[Y] = json!(["6000", "6000"]);
    machine[TRAVEL] = json!(["6000", "6000"]);
}

fn compare(input: &str, change_machine: impl FnOnce(&mut Value), expected: [&str; 3]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = crate::runner::repo_root().join(FIXTURES);
    let root = tempfile::Builder::new()
        .prefix("explicit-machine-limits-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    eprintln!("evidence: {root:?}");
    let model = root.join(input);
    let machine_path = root.join("machine.json");
    let process_path = root.join("process.json");
    let filament_path = root.join("filament.json");
    fs::copy(fixtures.join(input), &model).unwrap();
    fs::copy(fixtures.join("process.json"), &process_path).unwrap();
    fs::copy(fixtures.join("filament.json"), &filament_path).unwrap();
    let mut machine: Value =
        serde_json::from_slice(&fs::read(fixtures.join("machine.json")).unwrap()).unwrap();
    change_machine(&mut machine);
    fs::write(&machine_path, serde_json::to_vec(&machine).unwrap()).unwrap();

    let settings = crate::runner::join_paths([&machine_path, &process_path].into_iter());
    let list = [
        "--load-settings",
        &settings,
        "--load-filaments",
        path_str(&filament_path),
    ];
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    stages::run_live(
        runner.bin(),
        list.into_iter()
            .chain([
                "--arrange",
                "0",
                "--slice",
                "0",
                "--outputdir",
                path_str(&native_dir),
                path_str(&model),
            ])
            .map(str::to_owned),
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    for (key, expected) in [X, Y, TRAVEL].into_iter().zip(expected) {
        assert_footer(&native, key, expected);
    }
    // A zero cap leaves this Z acceleration unchanged.
    assert_footer(&native, "machine_max_acceleration_z", "1500,1500");

    let cli_path = root.join("ares.cli.gcode");
    let cli = cli_command()
        .args(["slice", "-o", path_str(&cli_path)])
        .args(list)
        .arg(&model)
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli = fs::read(cli_path).unwrap();
    let api = slice_flattened(
        (&fs::read(&model).unwrap(), input),
        &[machine_path, process_path],
        &[filament_path],
        &ares_core::ConfigOverrides::default(),
        ares_core::Placement::None,
    )
    .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    assert_eq!(api, cli, "public API/CLI bytes");
    golden::compare_ordered_bytes_generator_only(&native, &cli).unwrap();
}

fn assert_footer(bytes: &[u8], key: &str, expected: &str) {
    let prefix = format!("; {key} = ");
    let text = std::str::from_utf8(bytes).unwrap();
    assert_eq!(
        text.lines().find_map(|line| line.strip_prefix(&prefix)),
        Some(expected),
        "Native {key}"
    );
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap()
}
