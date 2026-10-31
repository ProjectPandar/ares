//! The legend Custom G-code overview (`GCodeViewer.cpp:4477-4546`) against the
//! live Native oracle of `tools/chain-probe/custom-gcode-overview.nix`: the
//! slice A probe's plate list and the real `LibVGCodeWrapper` conversion and
//! `libvgcode::Layers` of the Ares G-code, with the extracted overview kernel.
//! Failure inventory: the processor list or another plate; a ColorChange
//! label; f32 z comparison; `<=` against eps; a tie resolved to the lower
//! layer; z outside the layers; `-1` printed for an unmatched z; the time
//! summed through `layer`, in f64, for layer 0 or in stealth mode; sorting.
//! Public seams: `custom_gcode_overview` over `ViewerImpl::layers`.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, gcode_processor::CustomGCodeItem,
};
use ares_vgcode::{
    Layers, TimeMode, ViewerImpl, convert::from_processor_result, custom_gcode_overview,
};

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const TEST: &str = "native_rows_for_every_layer_match_case";
const OVERVIEW: &str = "ARES_CUSTOM_GCODE_OVERVIEW_BIN";
const PROBE: &str = "ARES_PLATE_CUSTOM_GCODES_BIN";
const EPSILON: f64 = 0.0011;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Ares G-code, plate list and layers of `project` (plate 1, key 0).
fn ares(project: &Path) -> (Vec<u8>, Vec<CustomGCodeItem>, Layers) {
    let bytes = std::fs::read(project).unwrap();
    let processed = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_processed(
            &bytes,
            GenerationMetadata::default(),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    let items = ares_core::load_project(&bytes)
        .unwrap()
        .curr_plate_custom_gcodes()
        .gcodes;
    let mut viewer = ViewerImpl::default();
    viewer.load(from_processor_result(&processed.result));
    (processed.gcode, items, viewer.layers().clone())
}

/// Live Native artifacts for `project` sliced by Ares as `gcode`.
fn native(project: &Path, gcode: &[u8]) -> Option<PathBuf> {
    native_environment::binary(OVERVIEW)?;
    native_environment::binary(PROBE)?;
    match Command::new("node").arg("--version").output() {
        Ok(output) => assert!(output.status.success(), "node --version failed"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            assert!(
                !native_environment::required(),
                "ARES_REQUIRE_ORCA=1: missing node; Native comparison not executed"
            );
            native_environment::skipped("node is unavailable");
            return None;
        }
        Err(error) => panic!("cannot launch node: {error}"),
    }
    let directory = tempfile::Builder::new()
        .prefix("custom-gcode-overview-live-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("live custom-gcode-overview artifacts: {directory:?}");
    let input = directory.join("ares.gcode");
    std::fs::write(&input, gcode).unwrap();
    let output = Command::new("node")
        .arg(repo().join("tools/chain-probe/custom-gcode-overview-fixtures.mjs"))
        .arg(directory.join("oracle"))
        .arg(project)
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "live custom-gcode-overview oracle failed: {} (artifacts {directory:?})",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(directory.join("oracle"))
}

fn rows(root: &Path, name: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root.join(name))
        .unwrap()
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

/// `(label, printed layer, time f32 bits)` per row.
fn overview(items: &[CustomGCodeItem], layers: &Layers) -> Vec<(String, String, u32)> {
    custom_gcode_overview(items, layers)
        .into_iter()
        .map(|row| {
            (
                row.label.to_owned(),
                row.layer.to_string(),
                row.time.to_bits(),
            )
        })
        .collect()
}

#[test]
fn required_missing_native_cannot_pass() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove(OVERVIEW)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_CUSTOM_GCODE_OVERVIEW_BIN"),
        "required failure did not identify the missing Native producer",
    );
}

#[test]
fn optional_missing_native_reports_skipped() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env_remove("ARES_REQUIRE_ORCA")
        .env_remove(OVERVIEW)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SKIPPED:"));
}

#[test]
fn native_rows_for_every_layer_match_case() {
    let project = repo().join("tests/parity/custom-gcode-overview/overview.3mf");
    let (gcode, items, layers) = ares(&project);
    let Some(root) = native(&project, &gcode) else {
        return;
    };
    let native_layers = rows(&root, "layers.tsv");
    let zs = layers.get_zs();
    assert_eq!(
        zs.iter()
            .zip(layers.get_times(TimeMode::Normal))
            .map(|(z, time)| vec![z.to_bits().to_string(), time.to_bits().to_string()])
            .collect::<Vec<_>>(),
        native_layers,
        "layer zs and Normal times"
    );

    // The plate list must reach every find_close_layer_idx case.
    let zs: Vec<f64> = zs.into_iter().map(f64::from).collect();
    let (first, last) = (zs[0], zs[zs.len() - 1]);
    // (distance to the lower layer, distance to the upper layer) strictly between two layers.
    let between = |z: f64| {
        zs.windows(2)
            .find(|w| w[0] < z && z < w[1])
            .map(|w| (z - w[0], w[1] - z))
    };
    let cases: [(&str, &dyn Fn(f64) -> bool); 8] = [
        ("exact layer z", &|z| zs.contains(&z)),
        ("within eps, upper nearer", &|z| {
            between(z).is_some_and(|(l, h)| h < l && h < EPSILON)
        }),
        ("within eps, lower nearer", &|z| {
            between(z).is_some_and(|(l, h)| l < h && l < EPSILON)
        }),
        ("equidistant between layers", &|z| {
            between(z).is_some_and(|(l, h)| l == h)
        }),
        ("below the first layer", &|z| first - z >= EPSILON),
        ("below the first layer within eps", &|z| {
            z < first && first - z < EPSILON
        }),
        ("above the last layer", &|z| z - last >= EPSILON),
        ("above the last layer within eps", &|z| {
            z > last && z - last < EPSILON
        }),
    ];
    for (case, matches) in cases {
        assert!(
            items.iter().any(|item| matches(item.print_z)),
            "overview: no item {case}"
        );
    }
    let native_rows = rows(&root, "rows.tsv");
    let labels: std::collections::BTreeSet<_> = native_rows.iter().map(|r| &r[0]).collect();
    assert_eq!(labels.len(), 5, "overview: every label, Unknown included");
    assert!(native_rows.iter().any(|r| r[1] == "1"), "overview: layer 0");
    assert!(
        native_rows.iter().any(|r| r[1] == "0"),
        "overview: no layer"
    );

    let expected: Vec<_> = native_rows
        .iter()
        .map(|r| (r[0].clone(), r[1].clone(), r[2].parse::<u32>().unwrap()))
        .collect();
    assert_eq!(
        overview(&items, &layers),
        expected,
        "overview: overview rows"
    );
}

#[test]
fn tick_free_project_has_no_rows_like_native() {
    let project = repo().join("tests/parity/orca_cli_ender3.3mf");
    let (gcode, items, layers) = ares(&project);
    let Some(root) = native(&project, &gcode) else {
        return;
    };
    assert!(rows(&root, "rows.tsv").is_empty(), "Native rows");
    assert!(overview(&items, &layers).is_empty());
}
