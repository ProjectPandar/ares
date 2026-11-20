#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::json;

use super::{
    cli_no_check::{gcode, run},
    cli_saved_load_settings_remap::{repack_project, replacement_preset},
    direct_stl_slice::block_stl,
};
use crate::{artifacts, load_settings_cli::preset_files, runner::OrcaRunner};

const ESCAPED_NOTE: &str = r#""First; line\n\"quote\"\\path\rend""#;

#[test]
fn loads_scalar_filament_colour_notes_and_ramming_parameters() {
    compare_saved_slice(false);
}

#[test]
fn loads_escaped_filament_notes_with_a_semicolon_in_the_first_member() {
    compare_saved_slice(true);
}

fn compare_saved_slice(escaped: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("preset-scalar-string-vectors-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let stl = root.join("block.stl");
    fs::write(&stl, block_stl(108.0)).unwrap();
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
        stl.to_str().unwrap(),
    ]
    .map(OsString::from);
    assert!(run(runner.bin(), &root, "export", &export).status.success());
    let saved = root.join("saved.3mf");
    repack_project(&root.join("export/cwd/out/saved.3mf"), &saved);

    let machine = replacement_preset(&presets[0], |profile| {
        profile["machine_start_gcode"] = json!(format!(
            "{}\nM117 Replacement machine",
            profile["machine_start_gcode"].as_str().unwrap()
        ));
    });
    let process = replacement_preset(&presets[1], |profile| {
        profile["wall_loops"] = json!("3");
        profile["timelapse_type"] = json!("1");
        profile["enable_prime_tower"] = json!("1");
        if escaped {
            profile["post_process"] = json!("");
        }
    });
    let filament = replacement_preset(&presets[2], |profile| {
        profile["filament_start_gcode"] = json!(["M117 Replacement filament"]);
        if escaped {
            profile["filament_notes"] = json!(format!("{ESCAPED_NOTE}; second"));
        }
    });
    let settings = format!("{machine};{process}");
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--arrange",
        "0",
        "--load-settings",
        &settings,
        "--load-filaments",
        &filament,
        saved.to_str().unwrap(),
    ]
    .map(OsString::from);
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let expected = gcode(&root.join("native/cwd"));
    let text = std::str::from_utf8(&expected).unwrap();
    let note = if escaped { ESCAPED_NOTE } else { r#""""# };
    assert!(
        text.lines()
            .any(|line| line == format!("; filament_notes = {note}")),
        "{root:?}"
    );
    assert!(
        text.lines()
            .any(|line| line == r#"; default_filament_colour = """#),
        "{root:?}"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(r#"; filament_ramming_parameters = "120 100 "#)),
        "{root:?}"
    );
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    assert_eq!(ares.status.code(), Some(0), "{root:?}");
    assert!(
        gcode(&root.join("ares/cwd")) == expected,
        "complete G-code differs: {root:?}"
    );
}
