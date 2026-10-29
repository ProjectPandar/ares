//! Shared runtime-only Native VD qualification for incident and default-repair tests.
use crate::native_environment;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

pub fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

pub fn first_difference(native: &Value, rust: &Value, path: &str) -> Option<Value> {
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

pub fn capture(case: &str) -> Option<(PathBuf, Value)> {
    native_environment::binary("ARES_ORCA_NATIVE_BIN")?;
    let resources = native_environment::value("ARES_ORCA_RESOURCES_DIR")?;
    assert!(
        Path::new(&resources).is_dir(),
        "configured Orca resources directory is missing"
    );
    native_environment::binary("ARES_ORCA_BIN")?;
    for key in [
        "ARES_ORCA_DEBUG_DIR",
        "ARES_ORCA_SOURCE_DIR",
        "ARES_ORCA_UPSTREAM_DIR",
    ] {
        let path = native_environment::value(key)?;
        assert!(
            Path::new(&path).is_dir(),
            "configured {key} directory is missing"
        );
    }
    native_environment::binary("ARES_GDB_BIN")?;
    let python = env::var_os("ARES_PYTHON_BIN")
        .expect("ARES_PYTHON_BIN is required inside the Orca parity environment");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let root = env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    fs::create_dir_all(&root).unwrap();
    let out = root.join(format!(
        "voronoi-incident-{case}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&out).unwrap();
    eprintln!("VORONOI_INCIDENT_ARTIFACT_ROOT={}", out.display());
    let output = Command::new(&python)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(repo.join("tests/parity/voronoi-incident/native.py"))
        .arg(&out)
        .arg("--case")
        .arg(case)
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
    let executable = env::current_exe().unwrap();
    save(
        &out.join("Rust-launch.json"),
        &json!({
            "pid":std::process::id(),"executable":executable,
            "binary_SHA256":Sha256::digest(fs::read(&executable).unwrap()).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "Native_completed_before_Rust":true,"HEAD":qualified["HEAD"],"argv":env::args().collect::<Vec<_>>()
        }),
    );
    Some((out, qualified))
}
