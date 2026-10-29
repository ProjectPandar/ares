//! `TreeSupport3D.cpp::merge_influence_areas` bucket-layout determinism
//! (SPEC `docs/architecture/2026-10-14-organic-merge-single-worker-spec.md`):
//! the same organic-support project must slice to byte-identical G-code
//! regardless of how many CPUs the slicer process may use.
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

fn first_allowed_cpu() -> String {
    let mask = Command::new("taskset")
        .arg("-pc")
        .arg(std::process::id().to_string())
        .output()
        .expect("taskset reports the test process affinity");
    assert!(mask.status.success(), "taskset -pc must succeed");
    let text = String::from_utf8(mask.stdout).expect("affinity mask is utf-8");
    let list = text
        .rsplit(':')
        .next()
        .expect("mask line follows the colon")
        .trim();
    let first = list
        .split(',')
        .map(|range| {
            range
                .split('-')
                .next()
                .expect("each mask item starts with a cpu number")
        })
        .min_by_key(|cpu| cpu.parse::<u32>().expect("cpu numbers are numeric"))
        .expect("the affinity mask is non-empty");
    first.to_owned()
}

fn slice_with_affinity(affinity: Option<&str>, output: &Path) -> Vec<u8> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/organic_merge_affinity/overhang-organic.3mf");
    let binary = env!("CARGO_BIN_EXE_ares");
    let status = match affinity {
        Some(cpu) => {
            let mut pinned = Command::new("taskset");
            pinned.arg("-c").arg(cpu).arg(binary);
            pinned
        }
        None => Command::new(binary),
    }
    .arg("slice")
    .arg("--plate")
    .arg("1")
    .arg("-o")
    .arg(output)
    .arg(&fixture)
    .output()
    .expect("the ares CLI slices the organic fixture");
    assert!(status.status.success(), "slicing must succeed");
    fs::read(output).expect("the G-code output exists")
}

#[test]
fn organic_support_output_is_identical_across_affinity_widths() {
    let directory = tempfile::tempdir().unwrap();
    let single = slice_with_affinity(
        Some(&first_allowed_cpu()),
        &directory.path().join("single.gcode"),
    );
    let inherited = slice_with_affinity(None, &directory.path().join("inherited.gcode"));
    let gcode = std::str::from_utf8(&inherited).expect("G-code is utf-8");
    assert!(
        gcode.contains(";TYPE:Support"),
        "the fixture must keep producing organic support so the comparison is not vacuous"
    );
    assert_eq!(single, inherited);
}
