use std::{
    fmt::Write,
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
fn binary_zero_declared_count_uses_actual_records() {
    compare_import("zero-count", binary(&tetrahedron(1.0), 0), true);
}

#[test]
fn binary_too_small_declared_count_uses_actual_records() {
    compare_import("small-count", binary(&tetrahedron(1.0), 1), true);
}

#[test]
fn binary_max_declared_count_uses_actual_records() {
    compare_import("max-count", binary(&tetrahedron(1.0), u32::MAX), true);
}

#[test]
fn ascii_high_bit_name_in_body_probe_matches_native_rejection() {
    let name = format!("{}é", "m".repeat(90));
    let input = ascii(&name, &tetrahedron(10.0));
    assert!(input[84..212].iter().any(|&byte| byte > 127));
    assert_ne!((input.len() - 84) % 50, 0);
    compare_import("ascii-probe-high-bit", input, false);
}

#[test]
fn ascii_high_bit_name_before_body_probe_preserves_import() {
    let input = ascii("mesh-é", &tetrahedron(10.0));
    assert!(input[84..212].iter().all(|&byte| byte <= 127));
    compare_import("ascii-header-high-bit", input, true);
}

#[test]
fn ascii_count_bytes_exceeding_wasm32_length_preserve_import() {
    let input = ascii("synthetic", &tetrahedron(200.0));
    let count = u32::from_le_bytes(input[80..84].try_into().unwrap());
    assert!(84 + u64::from(count) * 50 > u64::from(u32::MAX));
    compare_import("ascii-large-pseudo-count", input, true);
}

#[test]
fn binary_ascii_like_body_uses_length_fallback() {
    let input = binary(&tetrahedron(10.0), 4);
    assert!(input.starts_with(b"solid"));
    assert!(input[84..212].iter().all(|&byte| byte <= 127));
    compare_import("ascii-like-binary", input, true);
}

#[test]
fn binary_below_minimum_size_matches_native_rejection() {
    compare_import("below-minimum", binary(&tetrahedron(1.0)[..3], 3), false);
}

#[test]
fn binary_incomplete_record_matches_native_rejection() {
    let mut input = binary(&tetrahedron(1.0), 4);
    input.push(0);
    compare_import("incomplete-record", input, false);
}

#[test]
fn binary_short_body_probe_matches_native_rejection() {
    compare_import(
        "short-binary-probe",
        binary(&tetrahedron(1.0)[..2], 2),
        false,
    );
}

#[test]
fn ascii_short_body_probe_matches_native_rejection() {
    let input = ascii("short", &tetrahedron(10.0)[..1]);
    assert!(input.len() < 212);
    compare_import("short-ascii-probe", input, false);
}

fn compare_import(case: &str, input: Vec<u8>, admitted: bool) {
    let Some(native) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let root = PathBuf::from(
        std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
            .expect("set ARES_PARITY_ARTIFACT_ROOT to an external directory"),
    );
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("stl-format-{case}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("STL format artifacts: {}", directory.display());
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
    let expected = status.success().then(|| {
        let bytes = fs::read(exported.join(format!("obj_1_{case}.stl"))).unwrap();
        raw_binary_facets(&bytes)
    });
    let model = load_model(&input);
    let actual = model.as_ref().ok().map(|model| {
        model
            .triangles()
            .iter()
            .map(|triangle| triangle.vertices().map(|point| [point.x, point.y, point.z]))
            .collect::<Vec<Facet>>()
    });
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case,
            "comparator": "same admission; exact ordered raw facet vertices without sorting or transforms",
            "native_exit_code": status.code(),
            "native_status": status.to_string(),
            "required_admission": admitted,
            "native_facets": expected,
            "ares_facets": actual,
            "ares_error": model.as_ref().err().map(ToString::to_string),
            "equal": actual == expected,
            "scope": "public load_model import; no slicing or G-code parity claim",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(status.code().is_some(), "Native crashed: {status}");
    assert_eq!(status.success(), admitted, "Native admission: {case}");
    assert_eq!(actual, expected, "STL import differs from Native: {case}");
}

fn raw_binary_facets(input: &[u8]) -> Vec<Facet> {
    assert_eq!(input.len(), 84 + input[84..].len() / 50 * 50);
    assert_eq!(
        input[84..].len() / 50,
        u32::from_le_bytes(input[80..84].try_into().unwrap()) as usize
    );
    input[84..]
        .chunks_exact(50)
        .map(|record| {
            std::array::from_fn(|vertex| {
                std::array::from_fn(|axis| {
                    let start = 12 + vertex * 12 + axis * 4;
                    f32::from_le_bytes(record[start..start + 4].try_into().unwrap())
                })
            })
        })
        .collect()
}

fn tetrahedron(scale: f32) -> [Facet; 4] {
    let origin = [0.0; 3];
    let x = [scale, 0.0, 0.0];
    let y = [0.0, scale, 0.0];
    let z = [0.0, 0.0, scale];
    [[origin, y, x], [origin, x, z], [origin, z, y], [x, y, z]]
}

fn binary(facets: &[Facet], count: u32) -> Vec<u8> {
    let mut input = vec![0; 80];
    input[..5].copy_from_slice(b"solid");
    input.extend_from_slice(&count.to_le_bytes());
    for facet in facets {
        input.extend_from_slice(&[0; 12]);
        for vertex in facet {
            for coordinate in vertex {
                input.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        input.extend_from_slice(&[0; 2]);
    }
    input
}

fn ascii(name: &str, facets: &[Facet]) -> Vec<u8> {
    let mut input = format!("solid {name}\n");
    for facet in facets {
        input.push_str("facet normal 0 0 0\nouter loop\n");
        for [x, y, z] in facet {
            writeln!(input, "vertex {x} {y} {z}").unwrap();
        }
        input.push_str("endloop\nendfacet\n");
    }
    writeln!(input, "endsolid {name}").unwrap();
    input.into_bytes()
}
