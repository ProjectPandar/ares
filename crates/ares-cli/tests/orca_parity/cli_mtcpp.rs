#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use super::direct_stl_slice::block_stl;
use crate::{
    artifacts, load_settings_cli::banner_normalized, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_mtcpp_accepts_plate_below_limit() {
    compare((25, 1, false), true, 0);
}

#[test]
fn cli_mtcpp_accepts_plate_at_limit() {
    compare((24, 1, false), true, 0);
}

#[test]
fn cli_mtcpp_rejects_plate_when_cumulative_count_exceeds_limit() {
    compare((23, 1, false), true, 197);
}

#[test]
fn cli_mtcpp_zero_leaves_plate_unlimited() {
    compare((0, 0, false), true, 0);
}

#[test]
fn cli_mtcpp_rejects_stl_without_loaded_presets() {
    compare((11, 0, false), false, 197);
}

#[test]
fn cli_mtcpp_after_slice_does_not_limit_the_plate() {
    compare((23, 1, true), true, 0);
}

fn compare(options: (i32, u32, bool), presets: bool, exit: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-mtcpp-{}-", options.0))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    write_fixture(&root, presets);
    let native = run(runner.bin(), &root, "native", options, presets);
    let ares = run(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        &root,
        "ares",
        options,
        presets,
    );
    assert_eq!(native.exit, Some(exit), "{root:?}");
    assert!(native.stderr.is_empty(), "{root:?}");
    let expected = if exit == 0 {
        &["plate_1.gcode", "result.json"][..]
    } else {
        &["result.json"][..]
    };
    assert_eq!(native.inventory, expected, "{root:?}");
    let report: Value = serde_json::from_slice(&native.report).unwrap();
    if exit == 0 {
        assert_eq!(report["sliced_plates"][0]["triangle_count"], 24, "{root:?}");
    } else {
        assert_eq!(report["return_code"], -59, "{root:?}");
        assert_eq!(report["plate_index"], 1, "{root:?}");
        assert!(report.get("sliced_plates").is_none(), "{root:?}");
    }
    assert_eq!(ares.exit, native.exit, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.inventory, native.inventory, "{root:?}");
    assert_eq!(ares.report, native.report, "result.json differs: {root:?}");
    assert!(
        ares.gcode == native.gcode,
        "plate_1.gcode differs: {root:?}"
    );
}

fn write_fixture(root: &Path, presets: bool) {
    fs::write(root.join("first.stl"), block_stl(70.0)).unwrap();
    if !presets {
        return;
    }
    fs::write(root.join("second.stl"), block_stl(110.0)).unwrap();
    let write = |name: &str, value: Value| {
        fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    write(
        "machine.json",
        json!({
            "type": "machine", "name": "Synthetic single nozzle", "from": "system",
            "printer_technology": "FFF", "printable_height": "200",
            "printable_area": ["0x0", "200x0", "200x200", "0x200"],
            "nozzle_diameter": ["0.4"], "printer_extruder_id": ["1"],
            "printer_extruder_variant": ["Direct Drive Standard"], "layer_change_gcode": "G92 E0"
        }),
    );
    write(
        "process.json",
        json!({
            "type": "process", "name": "Synthetic walls", "from": "system",
            "compatible_printers": ["Synthetic single nozzle"], "layer_height": "0.2",
            "initial_layer_print_height": "0.2", "sparse_infill_density": "0%",
            "top_shell_layers": "0", "bottom_shell_layers": "0", "enable_prime_tower": "0"
        }),
    );
    write(
        "filament.json",
        json!({
            "type": "filament", "name": "Synthetic PLA", "from": "system", "filament_id": "SYNTH_PLA",
            "filament_diameter": ["1.75"], "filament_is_support": ["0"],
            "filament_extruder_variant": ["Direct Drive Standard"]
        }),
    );
}

struct Artifacts {
    exit: Option<i32>,
    stderr: Vec<u8>,
    inventory: Vec<String>,
    report: Vec<u8>,
    gcode: Option<Vec<u8>>,
}

fn run(
    bin: &Path,
    root: &Path,
    engine: &str,
    (limit, selection, after_slice): (i32, u32, bool),
    presets: bool,
) -> Artifacts {
    let cwd = root.join(engine);
    let out = cwd.join("out");
    let temp = cwd.join("tmp");
    fs::create_dir_all(&out).unwrap();
    fs::create_dir_all(&temp).unwrap();
    let mut command = Command::new(bin);
    command.current_dir(&cwd).env("TMPDIR", temp);
    let limit = limit.to_string();
    if !after_slice {
        command.args(["--mtcpp", &limit]);
    }
    command.args(["--slice", &selection.to_string(), "--outputdir", "out"]);
    if after_slice {
        command.args(["--mtcpp", &limit]);
    }
    if presets {
        command
            .args(["--arrange", "0", "--load-settings"])
            .arg(format!(
                "{};{}",
                root.join("machine.json").display(),
                root.join("process.json").display()
            ))
            .arg("--load-filaments")
            .arg(root.join("filament.json"))
            .arg(root.join("second.stl"));
    }
    let result = command.arg(root.join("first.stl")).output().unwrap();
    let exit = result.status.code();
    fs::write(cwd.join("exit"), format!("{exit:?}\n")).unwrap();
    fs::write(cwd.join("stdout"), &result.stdout).unwrap();
    fs::write(cwd.join("stderr"), &result.stderr).unwrap();
    let mut inventory = fs::read_dir(&out)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    inventory.sort_unstable();
    Artifacts {
        exit,
        stderr: result.stderr,
        inventory,
        report: fs::read(out.join("result.json"))
            .map(|bytes| result_comparison_bytes(&bytes))
            .unwrap_or_default(),
        gcode: fs::read(out.join("plate_1.gcode"))
            .ok()
            .map(|bytes| banner_normalized(&bytes)),
    }
}
