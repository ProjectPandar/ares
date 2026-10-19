//! Runtime Orca MMU VD -> unchanged workspace backend; no Native goldens.

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

#[path = "voronoi_incident/diagram.rs"]
mod diagram;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{env, fs, path::Path, process::Command};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn first_difference(native: &Value, rust: &Value, path: &str) -> Option<Value> {
    match (native, rust) {
        (Value::Object(a), Value::Object(b)) if a.keys().eq(b.keys()) => {
            for (key, value) in a {
                if let Some(diff) = first_difference(value, &b[key], &format!("{path}.{key}")) {
                    return Some(diff);
                }
            }
            None
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(diff) = first_difference(a, b, &format!("{path}[{index}]")) {
                    return Some(diff);
                }
            }
            None
        }
        _ if native == rust => None,
        _ => Some(json!({"path":path,"Native":native,"Rust":rust})),
    }
}

#[test]
fn runtime_native_rectangle_complete_ordered_diagram_matches() {
    let Some(_native) = native_environment::binary("ARES_ORCA_NATIVE_BIN") else {
        return;
    };
    let Some(resources) = native_environment::value("ARES_ORCA_RESOURCES_DIR") else {
        return;
    };
    assert!(
        Path::new(&resources).is_dir(),
        "configured Orca resources directory is missing"
    );
    let Some(_wrapper) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    for key in [
        "ARES_ORCA_DEBUG_DIR",
        "ARES_ORCA_SOURCE_DIR",
        "ARES_ORCA_UPSTREAM_DIR",
    ] {
        let Some(path) = native_environment::value(key) else {
            return;
        };
        assert!(
            Path::new(&path).is_dir(),
            "configured {key} directory is missing"
        );
    }
    if native_environment::binary("ARES_GDB_BIN").is_none() {
        return;
    }
    let python = env::var_os("ARES_PYTHON_BIN")
        .expect("ARES_PYTHON_BIN is required inside the Orca parity environment");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut builder = tempfile::Builder::new();
    builder.prefix("voronoi-incident-");
    let dir = if let Some(root) = env::var_os("ARES_PARITY_ARTIFACT_ROOT") {
        fs::create_dir_all(&root).unwrap();
        builder.tempdir_in(root).unwrap()
    } else {
        builder.tempdir().unwrap()
    };
    let out = dir.keep();
    eprintln!("VORONOI_INCIDENT_ARTIFACT_ROOT={}", out.display());
    let script = repo.join("tests/parity/voronoi-incident/native.py");
    let output = Command::new(&python)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(&script)
        .arg(&out)
        .output()
        .unwrap();
    fs::write(out.join("native-driver.stdout"), &output.stdout).unwrap();
    fs::write(out.join("native-driver.stderr"), &output.stderr).unwrap();
    save(
        &out.join("native-driver-exit.json"),
        &json!({"exit":output.status.code()}),
    );
    assert!(
        output.status.success(),
        "Native infrastructure NON-GATE; see {}",
        out.display()
    );
    let qualified = read_json(&out.join("Native-qualified.json"));
    assert_eq!(qualified["status"], "COMPLETE_NATIVE_FIRST");
    assert_eq!(qualified["completed_processes"], 2);
    assert_eq!(qualified["qualified_diagrams"], 100);
    let executable = env::current_exe().unwrap();
    save(
        &out.join("Rust-launch.json"),
        &json!({
            "pid":std::process::id(),"executable":executable,
            "binary_SHA256":Sha256::digest(fs::read(&executable).unwrap()).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "Native_completed_before_Rust":true,
            "HEAD":qualified["HEAD"],"argv":env::args().collect::<Vec<_>>()
        }),
    );
    let mut results = Vec::new();
    let mut first = None;
    for relative in qualified["records"].as_array().unwrap() {
        let relative = relative.as_str().unwrap();
        let row = read_json(&out.join(relative));
        let rust = diagram::build(&row["lines"]);
        let native = json!({
            "cells":row["diagram"]["cells"],
            "vertices":row["diagram"]["vertices"],
            "edges":row["diagram"]["edges"]
        });
        let difference = first_difference(&native, &rust, "$");
        if first.is_none() {
            first.clone_from(&difference);
        }
        let folder = out.join(relative).parent().unwrap().to_path_buf();
        save(&folder.join("Rust.diagram.json"), &rust);
        results.push(
            json!({"Native_record":relative,"first_difference":difference,"equal":native==rust}),
        );
    }
    assert_eq!(results.len(), 100);
    save(
        &out.join("comparison.json"),
        &json!({
            "status":if first.is_some() { "STRICT_DIAGRAM_MISMATCH" } else { "STRICT_DIAGRAM_MATCH" },
            "first_difference":first,"results":results,
            "no_sort_filter_tolerance_or_anchor_normalization":true,
            "scope":"one actual rectangle geometry, not full repaired VD equivalence"
        }),
    );
    assert!(
        first.is_none(),
        "strict complete ordered diagram mismatch: {first:?}; artifacts={}",
        out.display()
    );
}
