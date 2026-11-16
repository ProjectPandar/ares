use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn compare_context_field(key: &str, field: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = Command::new("git")
        .args(["-C"])
        .arg(repo.join("OrcaSlicer"))
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(source.status.success());
    assert_eq!(
        String::from_utf8(source.stdout).unwrap().trim(),
        "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19"
    );
    let root = std::env::var_os("ARES_CONFIGDEF_CONTEXT_ARTIFACTS")
        .expect("set ARES_CONFIGDEF_CONTEXT_ARTIFACTS to a fresh directory");
    let artifacts = Path::new(&root).join(key);
    fs::create_dir_all(&artifacts).unwrap();
    let output = artifacts.join("generated-configdef.json");
    let result = Command::new("node")
        .arg(repo.join("scripts/gen-configdef.mjs"))
        .arg("--dump")
        .stdout(Stdio::from(File::create(&output).unwrap()))
        .output()
        .unwrap();
    fs::write(artifacts.join("generator-stderr.log"), &result.stderr).unwrap();
    assert!(result.status.success(), "{:?}", result.status);
    let expected: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let public_bytes = ares_core::print_config_def_json();
    fs::write(artifacts.join("public-metadata.json"), public_bytes).unwrap();
    let actual: Value = serde_json::from_slice(public_bytes).unwrap();
    let source_entry = expected["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["key"] == key)
        .unwrap();
    let public_entry = actual["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["key"] == key)
        .unwrap();
    fs::write(
        artifacts.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "source": "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19",
            "key": key, "field": field,
            "expected": source_entry[field], "actual": public_entry[field],
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(public_entry[field], source_entry[field], "{key}.{field}");
}

macro_rules! context_cases {
    ($($test:ident: $key:literal, $field:literal;)*) => {$(
        #[test]
        fn $test() {
            compare_context_field($key, $field);
        }
    )*};
}

context_cases! {
    fan_kickstart: "fan_kickstart", "sidetext";
    fan_speedup_time: "fan_speedup_time", "sidetext";
    filament_toolchange_delay: "filament_toolchange_delay", "sidetext";
    input_shaping_damp_x: "input_shaping_damp_x", "label";
    input_shaping_damp_y: "input_shaping_damp_y", "label";
    input_shaping_freq_x: "input_shaping_freq_x", "label";
    input_shaping_freq_y: "input_shaping_freq_y", "label";
    machine_load_filament_time: "machine_load_filament_time", "sidetext";
    machine_tool_change_time: "machine_tool_change_time", "sidetext";
    machine_unload_filament_time: "machine_unload_filament_time", "sidetext";
    preheat_time: "preheat_time", "sidetext";
    slow_down_layer_time: "slow_down_layer_time", "sidetext";
    machine_bed_mass_y: "machine_bed_mass_Y", "sidetext";
    machine_max_printed_mass: "machine_max_printed_mass", "sidetext";
    machine_max_force_y: "machine_max_force_Y", "sidetext";
    machine_hotend_change_time: "machine_hotend_change_time", "sidetext";
}
