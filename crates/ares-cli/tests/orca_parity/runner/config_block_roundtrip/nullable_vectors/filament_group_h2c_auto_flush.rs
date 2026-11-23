use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use crate::OrcaRunner;

// Saved stock H2C 0.4 settings with one synthetic 10 mm cube and one PLA slot.
const PROJECT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/filament-group-flush-mechanisms/h2c_auto_flush_one_filament.3mf"
));
const ARGV: [&str; 5] = ["--slice", "1", "--outputdir", "out", "input.3mf"];

#[test]
fn h2c_auto_flush_uses_the_same_physical_nozzle_as_native() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let root = runner.work_dir();
    for (name, update, expected) in [
        ("stock-counts", json!({}), ["2", "0", "1"]),
        (
            "master-one",
            json!({"master_extruder_id": "1"}),
            ["1", "0", "0"],
        ),
        (
            "one-nozzle-per-extruder",
            json!({"extruder_max_nozzle_count": ["1", "1"]}),
            ["2", "0", "1"],
        ),
    ] {
        let case = prepare(root, name, PROJECT, update);
        let native = slice(runner.bin(), &case, "native");
        let actual = slice(Path::new(env!("CARGO_BIN_EXE_ares")), &case, "ares");
        assert_eq!(maps(&native), expected, "{name}: Native assignment");
        assert_eq!(maps(&actual), maps(&native), "{name}: Ares assignment");
    }
}

#[test]
fn h2c_auto_flush_rejects_detailed_multi_nozzle_inventory() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let case = prepare(
        runner.work_dir(),
        "reported-nozzles",
        PROJECT,
        json!({"extruder_nozzle_stats": ["Standard#1", "Standard#6"]}),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(ARGV)
        .current_dir(&case)
        .output()
        .unwrap();
    fs::write(case.join("ares.stderr"), &output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unsupported automatic filament grouping context")
    );
    assert!(!case.join("out/plate_1.gcode").exists());
}

fn prepare(root: &Path, name: &str, source: &[u8], update: Value) -> std::path::PathBuf {
    let case = root.join(name);
    fs::create_dir_all(case.join("out")).unwrap();
    let project = super::input::settings_variant(source, update.as_object().unwrap());
    fs::write(case.join("input.3mf"), project).unwrap();
    fs::write(case.join("argv.txt"), ARGV.join("\n") + "\n").unwrap();
    case
}

fn slice(bin: &Path, cwd: &Path, producer: &str) -> String {
    let output = Command::new(bin)
        .args(ARGV)
        .current_dir(cwd)
        .output()
        .unwrap();
    fs::write(cwd.join(format!("{producer}.stdout")), &output.stdout).unwrap();
    fs::write(cwd.join(format!("{producer}.stderr")), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{producer} in {}: {}",
        cwd.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let gcode = cwd.join(format!("{producer}.gcode"));
    fs::rename(cwd.join("out/plate_1.gcode"), &gcode).unwrap();
    fs::read_to_string(gcode).unwrap()
}

fn maps(gcode: &str) -> [&str; 3] {
    let block = gcode.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    ["filament_map", "filament_volume_map", "filament_nozzle_map"].map(|key| {
        block
            .lines()
            .find_map(|line| {
                let (name, value) = line.strip_prefix("; ")?.split_once(" = ")?;
                (name == key).then_some(value)
            })
            .unwrap()
    })
}
