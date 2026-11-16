use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn compare_metadata(key: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = std::env::var_os("ARES_ENUM_ARTIFACTS")
        .expect("set ARES_ENUM_ARTIFACTS to a fresh directory");
    let work = Path::new(&root).join(key);
    fs::create_dir_all(&work).unwrap();
    let source = Command::new("git")
        .arg("-C")
        .arg(repo.join("OrcaSlicer"))
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(source.status.success());
    assert_eq!(
        String::from_utf8(source.stdout).unwrap().trim(),
        "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19"
    );
    let output = work.join("generated-configdef.json");
    let result = Command::new("node")
        .arg(repo.join("scripts/gen-configdef.mjs"))
        .arg("--dump")
        .stdout(Stdio::from(File::create(&output).unwrap()))
        .output()
        .unwrap();
    fs::write(work.join("generator-stderr.log"), &result.stderr).unwrap();
    assert!(result.status.success());
    let generated: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let public = ares_core::print_config_def_json();
    fs::write(work.join("public-metadata.json"), public).unwrap();
    let actual: Value = serde_json::from_slice(public).unwrap();
    let source = generated["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["key"] == key)
        .unwrap();
    let observed = actual["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["key"] == key)
        .unwrap();
    fs::write(
        work.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({"key": key, "expected": source, "actual": observed}))
            .unwrap(),
    )
    .unwrap();
    for field in [
        "enum_values",
        "enum_labels",
        "enum_keys",
        "label",
        "full_label",
        "tooltip",
        "sidetext",
        "category",
    ] {
        assert_eq!(observed[field], source[field], "{key}.{field}");
    }
}

macro_rules! metadata_cases {
    ($($key:ident;)*) => {$(
        #[test]
        fn $key() {
            compare_metadata(stringify!($key));
        }
    )*};
}

metadata_cases! {
    bottom_surface_pattern;
    brim_type;
    filament_map_mode;
    host_type;
    internal_solid_infill_pattern;
    ironing_type;
    nozzle_type;
    print_order;
    printer_structure;
    sparse_infill_pattern;
    support_interface_pattern;
    top_surface_pattern;
}
