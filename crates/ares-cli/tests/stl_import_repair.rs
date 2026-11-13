use std::{
    fs::{self, File},
    path::PathBuf,
    process::Command,
};

use ares_core::load_model;
use serde_json::json;
use sha2::{Digest, Sha256};

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

type Facet = [[f32; 3]; 3];

#[test]
fn outward_ascii_import_matches_native() {
    compare_import("outward-ascii");
}

#[test]
fn outward_binary_import_matches_native() {
    compare_import("outward-binary");
}

#[test]
fn inward_ascii_import_matches_native() {
    compare_import("inward-ascii");
}

#[test]
fn inward_binary_import_matches_native() {
    compare_import("inward-binary");
}

#[test]
fn wrong_authored_ascii_import_matches_native() {
    compare_import("wrong-authored-ascii");
}

#[test]
fn wrong_authored_binary_import_matches_native() {
    compare_import("wrong-authored-binary");
}

fn compare_import(case: &str) {
    let Some(native) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let root = PathBuf::from(
        std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
            .expect("set ARES_PARITY_ARTIFACT_ROOT to an external directory"),
    );
    assert!(root.is_absolute());
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("stl-import-{case}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("STL import artifacts: {}", directory.display());
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/stl-closed-import-repair")
        .join(format!("{case}.stl"));
    let input = fs::read(fixture).unwrap();
    let input_path = directory.join(format!("{case}.stl"));
    fs::write(&input_path, &input).unwrap();
    let exported = directory.join("exported");
    let output = directory.join("output");
    fs::create_dir(&exported).unwrap();
    fs::create_dir(&output).unwrap();
    let appdir = directory.join("AppDir");
    let mut command = Command::new(&native);
    command
        .args(["--debug", "5", "--datadir"])
        .arg(directory.join("data"))
        .args(["--arrange", "0", "--export-stls"])
        .arg(&exported)
        .arg("--outputdir")
        .arg(&output)
        .arg(&input_path)
        .env("ORCA_JOB_APPDIR", &appdir)
        .env("OMP_NUM_THREADS", "1")
        .env("OPENBLAS_NUM_THREADS", "1")
        .current_dir(&directory)
        .stdout(File::create(directory.join("native.stdout")).unwrap())
        .stderr(File::create(directory.join("native.stderr")).unwrap());
    fs::write(
        directory.join("command.json"),
        serde_json::to_vec_pretty(&json!({
            "program": native,
            "argv": command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
            "cwd": directory,
            "environment": {"ORCA_JOB_APPDIR": appdir, "OMP_NUM_THREADS": "1", "OPENBLAS_NUM_THREADS": "1"},
            "input_sha256": Sha256::digest(&input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        }))
        .unwrap(),
    )
    .unwrap();
    let status = command.status().unwrap();
    fs::write(
        directory.join("native-status.json"),
        serde_json::to_vec_pretty(
            &json!({"exit_code": status.code(), "status": status.to_string()}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(status.success(), "Native STL export failed: {status}");
    let native_bytes = fs::read(exported.join(format!("obj_1_{case}.stl"))).unwrap();
    let expected = raw_binary_facets(&native_bytes);
    let model = load_model(&input).unwrap();
    let actual: Vec<Facet> = model
        .triangles()
        .iter()
        .map(|triangle| triangle.vertices().map(|point| [point.x, point.y, point.z]))
        .collect();
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case,
            "comparator": "exact ordered raw facet vertices, no sorting or transforms",
            "native_export_sha256": Sha256::digest(&native_bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "expected_native_facets": expected,
            "actual_ares_facets": actual,
            "equal": actual == expected,
            "scope": "public load_model geometry; no slicing or G-code parity claim",
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual, expected, "STL import differs for {case}");
}

fn raw_binary_facets(input: &[u8]) -> Vec<Facet> {
    assert!(input.len() >= 84, "Native export is not a binary STL");
    let count = usize::try_from(u32::from_le_bytes(input[80..84].try_into().unwrap())).unwrap();
    assert_eq!(input.len(), 84 + count * 50);
    input[84..]
        .as_chunks::<50>()
        .0
        .iter()
        .map(|facet| {
            std::array::from_fn(|vertex| {
                std::array::from_fn(|axis| {
                    let offset = 12 + vertex * 12 + axis * 4;
                    f32::from_le_bytes(facet[offset..offset + 4].try_into().unwrap())
                })
            })
        })
        .collect()
}
