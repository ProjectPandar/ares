#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::json;

use super::{
    cli_no_check::{gcode, inventory, run},
    cli_saved_load_settings_remap::{repack_project, replacement_preset},
    direct_stl_slice::block_stl,
};
use crate::{
    artifacts, load_settings_cli::preset_files, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_enable_timelapse_omitted_disables_smooth_tower_after_remap() {
    compare(Some("1"), None, "0");
}

#[test]
fn cli_enable_timelapse_preserves_smooth_tower_after_remap() {
    compare(Some("1"), Some("--enable-timelapse"), "1");
}

#[test]
fn cli_enable_timelapse_does_not_preserve_traditional_tower_after_remap() {
    compare(Some("0"), Some("--enable-timelapse=1"), "0");
}

#[test]
fn cli_enable_timelapse_leaves_unmapped_saved_smooth_tower_unchanged() {
    compare(None, Some("--enable-timelapse"), "1");
}

fn compare(remap_mode: Option<&str>, flag: Option<&str>, tower: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-enable-timelapse-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let machine = replacement_preset(&presets[0], |config| {
        config["time_lapse_gcode"] = json!("M117 Snapshot mode {timelapse_type}");
    });
    let settings = format!("{machine};{}", presets[1]);
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
    assert!(
        run(runner.bin(), &root, "export", &export).status.success(),
        "{root:?}"
    );
    let project = root.join("saved.3mf");
    repack_project(&root.join("export/cwd/out/saved.3mf"), &project);

    let replacement_settings = remap_mode.map(|mode| {
        let process = replacement_preset(&presets[1], |config| {
            config["timelapse_type"] = json!(mode);
            config["enable_prime_tower"] = json!("1");
        });
        format!("{machine};{process}")
    });
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--arrange",
        "0",
    ]
    .into_iter()
    .chain(flag)
    .chain(
        replacement_settings
            .as_deref()
            .into_iter()
            .flat_map(|settings| ["--load-settings", settings, "--load-filaments", &presets[2]]),
    )
    .chain([project.to_str().unwrap()])
    .map(OsString::from)
    .collect::<Vec<_>>();
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();

    let native = run(runner.bin(), &root, "native", &argv);
    let native_cwd = root.join("native/cwd");
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let expected = gcode(&native_cwd);
    let native_text = std::str::from_utf8(&expected).unwrap();
    let mode = remap_mode.unwrap_or("1");
    for line in [
        format!("M117 Snapshot mode {mode}"),
        format!("; timelapse_type = {mode}"),
        format!("; enable_prime_tower = {tower}"),
    ] {
        assert!(
            native_text.lines().any(|actual| actual == line),
            "missing {line:?}: {root:?}"
        );
    }

    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let actual_cwd = root.join("ares/cwd");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&actual_cwd), inventory(&native_cwd), "{root:?}");
    let report =
        |cwd: &Path| result_comparison_bytes(&fs::read(cwd.join("out/result.json")).unwrap());
    assert!(
        report(&actual_cwd) == report(&native_cwd),
        "complete report differs: {root:?}"
    );
    assert!(
        gcode(&actual_cwd) == expected,
        "complete G-code differs: {root:?}"
    );
}
