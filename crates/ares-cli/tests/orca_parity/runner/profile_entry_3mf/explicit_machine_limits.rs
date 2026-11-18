use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
};

use serde_json::{Value, json};

use crate::{
    golden,
    runner::{OrcaRunner, application::cli_command, profile_entry::slice_flattened, stages},
};

const FIXTURES: &str = "tests/parity/cli-explicit-machine-limits";
const X: &str = "machine_max_acceleration_x";
const Y: &str = "machine_max_acceleration_y";
const TRAVEL: &str = "machine_max_acceleration_travel";

#[derive(Clone, Copy)]
enum Input {
    Stl,
    SameNameProject,
    RenamedProject,
}

#[test]
fn explicit_machine_limits_clamp_stock_stl() {
    compare(Input::Stl, |_| {}, ["6000,6000"; 3]);
}

#[test]
fn explicit_machine_limits_clamp_x_only_stl() {
    compare(Input::Stl, preclamp_y_and_travel, ["6000,6000"; 3]);
}

#[test]
fn explicit_machine_limits_keep_preclamped_stl() {
    compare(
        Input::Stl,
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
        Input::Stl,
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
        Input::SameNameProject,
        |_| {},
        ["20000,20000", "20000,20000", "9000,9000"],
    );
}

#[test]
fn explicit_machine_limits_clamp_renamed_3mf() {
    compare(Input::RenamedProject, |_| {}, ["6000,6000"; 3]);
}

fn preclamp_y_and_travel(machine: &mut Value) {
    machine[Y] = json!(["6000", "6000"]);
    machine[TRAVEL] = json!(["6000", "6000"]);
}

fn compare(input: Input, change_machine: impl FnOnce(&mut Value), expected: [&str; 3]) {
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
    let cube = root.join("cube10.stl");
    let machine_path = root.join("machine.json");
    let process_path = root.join("process.json");
    let filament_path = root.join("filament.json");
    fs::copy(fixtures.join("cube10.stl"), &cube).unwrap();
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
    let model = match input {
        Input::Stl => cube,
        Input::SameNameProject | Input::RenamedProject => {
            let project = root.join("a1mini.3mf");
            stages::run_live(
                runner.bin(),
                list.into_iter()
                    .chain([
                        "--arrange",
                        "1",
                        "--export-3mf",
                        path_str(&project),
                        path_str(&cube),
                    ])
                    .map(str::to_owned),
                stages::Stage::Export,
                &root,
            )
            .unwrap();
            if matches!(input, Input::RenamedProject) {
                let renamed = root.join("renamed.3mf");
                let bytes = fs::read(&project).unwrap();
                fs::write(&renamed, rename_embedded_printer(&bytes)).unwrap();
                renamed
            } else {
                project
            }
        }
    };
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
        (
            &fs::read(&model).unwrap(),
            model.file_name().unwrap().to_str().unwrap(),
        ),
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

fn rename_embedded_printer(project: &[u8]) -> Vec<u8> {
    let mut settings = crate::runner::application::settings(project).unwrap();
    settings.insert(
        "printer_settings_id".to_owned(),
        json!("Original printer name"),
    );

    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
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
