use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::{
    super::{cli_no_check::run, cli_saved_load_settings_remap::repack_project},
    compare, outputs,
};
use crate::{
    artifacts, load_settings_cli::preset_files, presets::VendorProfiles, runner::OrcaRunner,
};

#[test]
fn leading_empty_filament_keeps_the_first_slot_on_a_selected_plate() {
    verify("1", "AB", ";A", "1.24,1.24");
}

#[test]
fn leading_empty_filament_keeps_the_first_slot_on_all_plates() {
    verify("0", "AB", ";A", "1.24,1.24");
}

#[test]
fn internal_empty_filament_keeps_the_middle_slot_on_a_selected_plate() {
    verify("1", "BBA", "A;;B", "1.24,1.27,1.27");
}

#[test]
fn internal_empty_filament_keeps_the_middle_slot_on_all_plates() {
    verify("0", "BBA", "A;;B", "1.24,1.27,1.27");
}

#[test]
fn trailing_empty_filament_keeps_the_last_slot_on_a_selected_plate() {
    verify("1", "AB", "A;", "1.24,1.27");
}

#[test]
fn trailing_empty_filament_keeps_the_last_slot_on_all_plates() {
    verify("0", "AB", "A;", "1.24,1.27");
}

fn verify(selection: &str, embedded: &str, replacements: &str, density: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-filament-slots-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let petg = root.join("petg.json");
    write_petg(&petg);
    let file = |slot| match slot {
        'A' => presets[2].as_str(),
        'B' => petg.to_str().unwrap(),
        _ => unreachable!(),
    };
    let embedded = embedded.chars().map(file).collect::<Vec<_>>().join(";");
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
        &embedded,
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
    let replacements = replacements
        .split(';')
        .map(|slot| slot.chars().next().map_or("", file))
        .collect::<Vec<_>>()
        .join(";");
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        selection,
        "--arrange",
        "0",
        "--load-settings",
        &settings,
        "--load-filaments",
        &replacements,
        project.to_str().unwrap(),
    ]
    .map(OsString::from);
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let expected = outputs(&root.join("native/cwd"), true, &["plate_1.gcode"]);
    assert!(
        std::str::from_utf8(&expected.0[0])
            .unwrap()
            .lines()
            .any(|line| line.strip_prefix("; filament_density = ") == Some(density)),
        "{root:?}"
    );
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    let actual = outputs(&root.join("ares/cwd"), true, &["plate_1.gcode"]);
    compare(&expected, &actual, false, &root);
}

fn write_petg(path: &Path) {
    let profiles_root = crate::runner::repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = VendorProfiles::load(&profiles_root, "Creality").unwrap();
    let mut file = Value::Object(profiles.filament("CR-PETG @Ender-3-all").unwrap());
    file["type"] = "filament".into();
    file["from"] = "system".into();
    file["instantiation"] = "true".into();
    fs::write(path, serde_json::to_vec(&file).unwrap()).unwrap();
}
