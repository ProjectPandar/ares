use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::{
    super::{cli_no_check::run, cli_saved_load_settings_remap::repack_project},
    compare, outputs,
};
use crate::{artifacts, load_settings_cli::preset_files, runner::OrcaRunner};

#[test]
fn direct_settings_only_keeps_embedded_filament_without_mapping_it() {
    verify("1", true);
}

#[test]
fn direct_all_plates_accepts_a_process_without_filament_files() {
    verify("0", true);
}

#[test]
fn direct_filaments_only_keeps_embedded_machine_and_process() {
    verify("1", false);
}

#[test]
fn direct_all_plates_accepts_filaments_without_settings_files() {
    verify("0", false);
}

fn verify(selection: &str, settings_only: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-independent-presets-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let cube = root.join("cube.stl");
    fs::write(
        &cube,
        include_bytes!("../../../../../tests/parity/cube10.stl"),
    )
    .unwrap();
    let export = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--export-3mf",
        "saved.3mf",
        "--load-settings",
        &settings,
        "--load-filaments",
        &presets[2],
        cube.to_str().unwrap(),
    ]
    .map(OsString::from);
    assert_eq!(
        run(runner.bin(), &root, "export", &export).status.code(),
        Some(0),
        "{root:?}"
    );
    let project = root.join("saved.3mf");
    repack_project(&root.join("export/cwd/out/saved.3mf"), &project);

    let (source, flag) = if settings_only {
        (&presets[1], "--load-settings")
    } else {
        (&presets[2], "--load-filaments")
    };
    let mut replacement: Value = serde_json::from_slice(&fs::read(source).unwrap()).unwrap();
    if settings_only {
        replacement["outer_wall_speed"] = "25".into();
        replacement["enable_prime_tower"] = "1".into();
    } else {
        replacement["nozzle_temperature"] = serde_json::json!(["215"]);
        replacement["nozzle_temperature_initial_layer"] = serde_json::json!(["215"]);
    }
    let file = root.join("replacement.json");
    fs::write(&file, serde_json::to_vec(&replacement).unwrap()).unwrap();
    let files = if settings_only && selection == "1" {
        format!("{};{}", presets[0], file.display())
    } else {
        file.to_str().unwrap().to_owned()
    };
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        selection,
        "--arrange",
        "0",
        flag,
        &files,
        project.to_str().unwrap(),
    ]
    .map(OsString::from);
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let expected = outputs(&root.join("native/cwd"), true, &["plate_1.gcode"]);
    let text = std::str::from_utf8(&expected.0[0]).unwrap();
    let settings = if settings_only {
        [
            "outer_wall_speed = 25",
            "nozzle_temperature = 200",
            "different_settings_to_system = ;;",
        ]
    } else {
        [
            "outer_wall_speed = 40",
            "nozzle_temperature = 215",
            "different_settings_to_system = enable_prime_tower;;",
        ]
    };
    for setting in settings {
        assert!(
            text.lines()
                .any(|line| line.strip_prefix("; ") == Some(setting)),
            "{setting}: {root:?}"
        );
    }
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    let actual = outputs(&root.join("ares/cwd"), true, &["plate_1.gcode"]);
    compare(&expected, &actual, false, &root);
}
