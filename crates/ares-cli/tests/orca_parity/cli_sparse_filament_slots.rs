#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use crate::{
    artifacts, load_settings_cli::banner_normalized, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_sparse_filament_slots_preserve_leading_gap() {
    compare("leading", ";a.json;b.json", false, 0);
}

#[test]
fn cli_sparse_filament_slots_preserve_interior_gap() {
    compare("interior", "a.json;;b.json", false, 0);
}

#[test]
fn cli_sparse_filament_slots_preserve_trailing_gap_with_full_support_vector() {
    compare("trailing", "a.json;b.json;", true, 0);
}

#[test]
fn cli_sparse_filament_slots_reject_trailing_gap_with_short_support_vector() {
    compare("short-support", "a.json;b.json;", false, 251);
}

fn compare(label: &str, list: &str, padded_support: bool, exit: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-sparse-filament-slots-{label}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    write_fixture(&root, padded_support);
    let native = run(runner.bin(), &root, "native", list);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", list);
    assert_eq!(native.exit, Some(exit), "{root:?}");
    assert_eq!(ares.exit, native.exit, "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    let expected = if exit == 0 {
        &["plate_1.gcode", "result.json"][..]
    } else {
        &["result.json"][..]
    };
    assert_eq!(native.inventory, expected, "{root:?}");
    assert_eq!(ares.inventory, native.inventory, "{root:?}");
    assert!(
        ares.report == native.report,
        "result.json differs: {root:?}"
    );
    assert!(
        ares.gcode == native.gcode,
        "plate_1.gcode differs: {root:?}"
    );
}

fn write_fixture(root: &Path, padded_support: bool) {
    let write = |name: &str, value: Value| {
        fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    let mut machine = json!({
        "type": "machine", "name": "Synthetic single nozzle", "from": "system",
        "printer_technology": "FFF", "printable_height": "200",
        "printable_area": ["0x0", "200x0", "200x200", "0x200"],
        "nozzle_diameter": ["0.4"], "printer_extruder_id": ["1"],
        "printer_extruder_variant": ["Direct Drive Standard"], "layer_change_gcode": "G92 E0"
    });
    if padded_support {
        machine["filament_is_support"] = json!(["0", "0", "0"]);
        // Native sizes its flush minima from filament_diameter.
        machine["filament_diameter"] = json!(["1.75", "1.75", "1.75"]);
    }
    write("machine.json", machine);
    write(
        "process.json",
        json!({
            "type": "process", "name": "Synthetic walls", "from": "system",
            "compatible_printers": ["Synthetic single nozzle"], "layer_height": "0.2",
            "initial_layer_print_height": "0.2", "sparse_infill_density": "0%",
            "top_shell_layers": "0", "bottom_shell_layers": "0", "enable_prime_tower": "0"
        }),
    );
    for (file, name, id, density, temperature, gcode) in [
        (
            "a.json",
            "Synthetic A",
            "SYNTH_A",
            "1.11",
            "205",
            "; synthetic A",
        ),
        (
            "b.json",
            "Synthetic B",
            "SYNTH_B",
            "1.33",
            "225",
            "; synthetic B",
        ),
    ] {
        write(
            file,
            json!({
                "type": "filament", "name": name, "from": "system", "filament_id": id,
                "filament_diameter": ["1.75"], "filament_density": [density],
                "filament_is_support": ["0"], "nozzle_temperature": [temperature],
                "filament_extruder_variant": ["Direct Drive Standard"], "filament_start_gcode": [gcode]
            }),
        );
    }
    fs::copy(
        crate::runner::repo_root().join("tests/parity/cube10.stl"),
        root.join("cube10.stl"),
    )
    .unwrap();
}

struct Artifacts {
    exit: Option<i32>,
    stderr: Vec<u8>,
    inventory: Vec<String>,
    report: Vec<u8>,
    gcode: Option<Vec<u8>>,
}

fn run(bin: &Path, root: &Path, engine: &str, list: &str) -> Artifacts {
    let cwd = root.join(engine);
    let out = cwd.join("out");
    fs::create_dir_all(&out).unwrap();
    let list = list
        .split(';')
        .map(|name| {
            if name.is_empty() {
                String::new()
            } else {
                root.join(name).to_string_lossy().into_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(";");
    let result = Command::new(bin)
        .current_dir(&cwd)
        .args(["--slice", "1", "--outputdir", "out", "--load-settings"])
        .arg(format!(
            "{};{}",
            root.join("machine.json").display(),
            root.join("process.json").display()
        ))
        .args(["--load-filaments", &list])
        .arg(root.join("cube10.stl"))
        .output()
        .unwrap();
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
