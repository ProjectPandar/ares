//! `Layer::make_ironing` (`Fill.cpp:1707-1932`) routes `ironing_pattern` through
//! `Fill::new_from_type` (`FillBase.cpp:65-104`) — every `InfillPattern` reaches
//! the corresponding `fill_surface`. The `ironing.rs` `_` arm at
//! `fill_entities/ironing.rs:231-234` instead rejects every pattern outside the
//! ported set with `UnsupportedProjectFeature("ironing_pattern")`.
//!
//! Fixture `tests/parity/summary/time-cost.3mf`: real Arachne model with `stTop`
//! surfaces on a two-nozzle X2D; patching `ironing_type=top` + `ironing_pattern=<x>`
//! produces Native ironing feature blocks. The rectilinear control stays on it as the
//! whole-output dual-nozzle case; the other patterns run on the single-nozzle base of
//! `single_nozzle` (`docs/architecture/ironing-single-nozzle-legs-spec.md`). Probes
//! ran on OrcaSlicer v2.5.0-beta `345efec64`.
//!
//! Legs `adaptivecubic`/`supportcubic`/`lightning` upstream-SIGSEGV inside
//! `FillAdaptive::Filler::_fill_surface_single` /
//! `FillLightning::Filler::_fill_surface_single` — `make_ironing` never
//! supplies their precomputed octree/generator context. Ares keeps
//! `UnsupportedProjectFeature` for them; these legs assert the guard still
//! fires post-fix (no regression on the crash-boundary contract).
//!
//! `3dhoneycomb` has no defined Native result in `make_ironing`: `layer_height`
//! stays zero (`Fill.cpp:1814-1816`, `FillBase.hpp:91`), so `zScale` is non-finite
//! (`Fill3DHoneycomb.cpp:371-391`), and whether Native then crashes depends on
//! geometry. Ares rejects it; that test does not run Native.
//!
//! `concentricinternal` is a `internal_solid_infill_pattern`-only upstream
//! label (`FillBase.cpp:72`), not reachable via `ironing_pattern`.

#[path = "ironing_pattern/single_nozzle.rs"]
pub(super) mod single_nozzle;
#[path = "ironing_pattern/three_d_honeycomb_guard.rs"]
mod three_d_honeycomb_guard;

use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::PathBuf,
    process::Command,
};

const BASE: &[u8] = include_bytes!("../../../../../tests/parity/summary/time-cost.3mf");
const SETTINGS: &str = "Metadata/project_settings.config";
const PREFIX: &str = "ironing-pattern";

const ANCHOR_TYPE: &str = "\"ironing_type\": \"no ironing\"";
const ANCHOR_PATTERN: &str = "\"ironing_pattern\": \"rectilinear\"";
/// `GCode.cpp:5348-5353` — `; printing object <name> id:<hash> copy`
/// labels read `print_object.get_id()` (unstable `ModelObject` hash;
/// `id:0` on the seeded replay, ASLR-dependent on the official AppImage,
/// `initialized_object_id` precedent).
/// With `gcode_label_objects = 0`, upstream emits
/// `; start printing object, unique label id: <identify_id>`
/// (`GCode.cpp:5356-5365`), driven by `ModelInstance::identify_id` —
/// deterministic across oracles; no output masking needed.
const ANCHOR_LABELS: &str = "\"gcode_label_objects\": \"1\"";

struct Leg {
    /// Test suffix + tempdir prefix.
    label: &'static str,
    /// `ironing_pattern` value injected into `project_settings.config`.
    pattern: &'static str,
    /// `true` when upstream SIGSEGVs (AdaptiveCubic/SupportCubic/Lightning):
    /// Ares keeps `UnsupportedProjectFeature`; assert it, do not diff.
    native_crashes: bool,
}

const LEGS: [Leg; 4] = [
    Leg {
        label: "control-rectilinear",
        pattern: "rectilinear",
        native_crashes: false,
    },
    Leg {
        label: "adaptivecubic",
        pattern: "adaptivecubic",
        native_crashes: true,
    },
    Leg {
        label: "supportcubic",
        pattern: "supportcubic",
        native_crashes: true,
    },
    Leg {
        label: "lightning",
        pattern: "lightning",
        native_crashes: true,
    },
];

/// Rebuilds the `base` archive with its project configuration passed through `edit`.
fn rewrite_settings(base: &[u8], edit: impl FnOnce(String) -> String) -> Vec<u8> {
    let mut edit = Some(edit);
    let mut reader = zip::ZipArchive::new(Cursor::new(base)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == SETTINGS {
            let edit = edit.take().unwrap();
            bytes = edit(String::from_utf8(bytes).unwrap()).into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    assert!(edit.is_none(), "archive must carry {SETTINGS}");
    writer.finish().unwrap().into_inner()
}

/// Turns on top ironing with `pattern` in a configuration that carries the anchors.
fn iron_top(text: &str, pattern: &str) -> String {
    assert!(
        text.contains(ANCHOR_TYPE),
        "fixture must carry {ANCHOR_TYPE}"
    );
    assert!(
        text.contains(ANCHOR_PATTERN),
        "fixture must carry {ANCHOR_PATTERN}"
    );
    text.replacen(ANCHOR_TYPE, "\"ironing_type\": \"top\"", 1)
        .replacen(
            ANCHOR_PATTERN,
            &format!("\"ironing_pattern\": \"{pattern}\""),
            1,
        )
}

fn project(pattern: &str) -> Vec<u8> {
    rewrite_settings(BASE, |text| {
        assert!(
            text.contains(ANCHOR_LABELS),
            "fixture must carry {ANCHOR_LABELS}"
        );
        iron_top(&text, pattern).replacen(ANCHOR_LABELS, "\"gcode_label_objects\": \"0\"", 1)
    })
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `--slice 0 --outputdir` Native reference; requires exit0 + nonempty
/// gcode. Skipped for `native_crashes` legs (upstream SIGSEGV has no
/// reference to emit).
fn native(runner: &OrcaRunner, directory: &std::path::Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir_all(directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(
        !reference.is_empty(),
        "{directory:?}: native reference must be non-empty"
    );
    reference
}

/// One leg's artifact directories, input project and validated-API slice; `setting` is the
/// value under test.
pub(super) struct Case {
    label: &'static str,
    setting: &'static str,
    root: PathBuf,
    directory: PathBuf,
    project: Vec<u8>,
    public: Result<Vec<u8>, ares_core::SliceError>,
}

/// `None` when no Native runner is configured; artifacts go to `<prefix>-<label>-*`.
pub(super) fn prepare(
    (prefix, label, setting): (&str, &'static str, &'static str),
    project: Vec<u8>,
) -> Option<(OrcaRunner, Case)> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let root = tempfile::Builder::new()
        .prefix(&format!("{prefix}-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    fs::create_dir_all(&directory).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let case = Case {
        label,
        setting,
        root,
        directory,
        project,
        public,
    };
    Some((runner, case))
}

/// Upstream SIGSEGV on `FillAdaptive::Filler::_fill_surface_single` /
/// `FillLightning::Filler::_fill_surface_single` — assert Native
/// actually fails (signal/nonzero), and Ares keeps the
/// `UnsupportedProjectFeature` guard.
fn assert_native_crash_is_guarded(runner: &OrcaRunner, case: Case) {
    let directory = &case.directory;
    let path = directory.join("input.3mf");
    fs::write(&path, &case.project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    let native_result = stages::run(runner.bin(), args, stages::Stage::Slice, directory);
    let native_failure = native_result.expect_err(&format!(
        "{directory:?}: native must crash on `{}`, but exited 0",
        case.setting
    ));
    let (native_code, native_status) = native_failure
        .captured
        .as_ref()
        .map(|c| (c.code, c.status.clone()))
        .unwrap_or((None, String::new()));
    let error = case
        .public
        .expect_err("crash-class pattern must remain unsupported");
    assert!(
        format!("{error:?}").contains("ironing_pattern"),
        "{:?}: error must carry `ironing_pattern`, got {error:?}",
        directory
    );
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label": case.label,
            "setting": case.setting,
            "native_crashes": true,
            "native_failure": format!("{native_failure}"),
            "native_code": native_code,
            "native_status": native_status,
            "ares_error": format!("{error:?}"),
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Whole-output comparison of the validated API and the CLI against Native;
/// `premise` inspects the Native reference before any comparison.
pub(super) fn assert_matches_native(runner: &OrcaRunner, case: &Case, premise: impl FnOnce(&[u8])) {
    let Case {
        root,
        directory,
        public,
        ..
    } = case;
    let reference = native(runner, directory, &case.project);
    premise(&reference);
    if let Ok(gcode) = public {
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = crate::runner::application::CliCommand::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    let raw_equals_cli = match (public, fs::read(&output)) {
        (Ok(api), Ok(cli)) => api == &cli,
        _ => false,
    };
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label": case.label,
            "setting": case.setting,
            "input_sha256": sha(&case.project),
            "native_sha256": sha(&reference),
            "validated_api_comparison": api_comparison,
            "cli_comparison": cli_comparison,
            "raw_api_equals_cli": raw_equals_cli,
            "CLI_code": cli.status.code(),
            "api_error": public.as_ref().err().map(|error| format!("{error:?}")),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    assert!(raw_equals_cli, "{root:?}: raw API must equal CLI output");
}

fn run_leg(leg: &Leg) {
    let Some((runner, case)) = prepare((PREFIX, leg.label, leg.pattern), project(leg.pattern))
    else {
        return;
    };
    if leg.native_crashes {
        assert_native_crash_is_guarded(&runner, case);
    } else {
        assert_matches_native(&runner, &case, |_| {});
    }
}

macro_rules! legs {
    ($($name:ident => $index:expr,)*) => {$(#[test] fn $name() { run_leg(&LEGS[$index]); })*};
}

legs! {
    orca_parity_ironing_pattern_control_rectilinear => 0,
    orca_parity_ironing_pattern_adaptivecubic => 1,
    orca_parity_ironing_pattern_supportcubic => 2,
    orca_parity_ironing_pattern_lightning => 3,
}
