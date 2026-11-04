//! `Layer::make_ironing` (`Fill.cpp:1573-1705`) routes `ironing_pattern` through
//! `Fill::new_from_type` (`FillBase.cpp:40-77`) — every `InfillPattern` reaches
//! the corresponding `fill_surface`. The `ironing.rs` `_` arm at
//! `fill_entities/ironing.rs:231-234` instead rejects every pattern outside the
//! ported set with `UnsupportedProjectFeature("ironing_pattern")`.
//!
//! Fixture `tests/parity/summary/time-cost.3mf`: real Arachne model with `stTop`
//! surfaces; patching `ironing_type=top` + `ironing_pattern=<x>` produces
//! Native ironing feature blocks on every leg below. Probes ran on official
//! OrcaSlicer 2.4.2.
//!
//! Legs `adaptivecubic`/`supportcubic`/`lightning` upstream-SIGSEGV inside
//! `FillAdaptive::Filler::_fill_surface_single` /
//! `FillLightning::Filler::_fill_surface_single` — `make_ironing` never
//! supplies their precomputed octree/generator context. Ares keeps
//! `UnsupportedProjectFeature` for them; these legs assert the guard still
//! fires post-fix (no regression on the crash-boundary contract).
//!
//! `concentricinternal` is a `internal_solid_infill_pattern`-only upstream
//! label (`FillBase.cpp:72`), not reachable via `ironing_pattern`.

use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const BASE: &[u8] = include_bytes!("../../../../../tests/parity/summary/time-cost.3mf");

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

const LEGS: [Leg; 20] = [
    Leg {
        label: "control-rectilinear",
        pattern: "rectilinear",
        native_crashes: false,
    },
    Leg {
        label: "monotonic",
        pattern: "monotonic",
        native_crashes: false,
    },
    Leg {
        label: "monotonicline",
        pattern: "monotonicline",
        native_crashes: false,
    },
    Leg {
        label: "alignedrectilinear",
        pattern: "alignedrectilinear",
        native_crashes: false,
    },
    Leg {
        label: "line",
        pattern: "line",
        native_crashes: false,
    },
    Leg {
        label: "grid",
        pattern: "grid",
        native_crashes: false,
    },
    Leg {
        label: "triangles",
        pattern: "triangles",
        native_crashes: false,
    },
    Leg {
        label: "tri-hexagon",
        pattern: "tri-hexagon",
        native_crashes: false,
    },
    Leg {
        label: "cubic",
        pattern: "cubic",
        native_crashes: false,
    },
    Leg {
        label: "quartercubic",
        pattern: "quartercubic",
        native_crashes: false,
    },
    Leg {
        label: "honeycomb",
        pattern: "honeycomb",
        native_crashes: false,
    },
    Leg {
        label: "3dhoneycomb",
        pattern: "3dhoneycomb",
        native_crashes: false,
    },
    Leg {
        label: "lateral-honeycomb",
        pattern: "lateral-honeycomb",
        native_crashes: false,
    },
    Leg {
        label: "crosshatch",
        pattern: "crosshatch",
        native_crashes: false,
    },
    Leg {
        label: "tpmsd",
        pattern: "tpmsd",
        native_crashes: false,
    },
    Leg {
        label: "tpmsfk",
        pattern: "tpmsfk",
        native_crashes: false,
    },
    Leg {
        label: "gyroid",
        pattern: "gyroid",
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

fn project(leg: &Leg) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut text = String::from_utf8(bytes).unwrap();
            assert!(
                text.contains(ANCHOR_TYPE),
                "fixture must carry {ANCHOR_TYPE}"
            );
            assert!(
                text.contains(ANCHOR_PATTERN),
                "fixture must carry {ANCHOR_PATTERN}"
            );
            assert!(
                text.contains(ANCHOR_LABELS),
                "fixture must carry {ANCHOR_LABELS}"
            );
            text = text.replacen(ANCHOR_TYPE, "\"ironing_type\": \"top\"", 1);
            text = text.replacen(
                ANCHOR_PATTERN,
                &format!("\"ironing_pattern\": \"{}\"", leg.pattern),
                1,
            );
            text = text.replacen(ANCHOR_LABELS, "\"gcode_label_objects\": \"0\"", 1);
            bytes = text.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
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

fn run_leg(leg: &Leg) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("ironing-pattern-{}-", leg.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    fs::create_dir_all(&directory).unwrap();
    let project = project(leg);

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

    if leg.native_crashes {
        // Upstream SIGSEGV on `FillAdaptive::Filler::_fill_surface_single` /
        // `FillLightning::Filler::_fill_surface_single` — assert Native
        // actually fails (signal/nonzero), and Ares keeps the
        // `UnsupportedProjectFeature` guard.
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("input.3mf");
        fs::write(&path, &project).unwrap();
        let output = directory.join("native");
        fs::create_dir(&output).unwrap();
        let args = [
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            output.to_str().unwrap().into(),
            path.to_str().unwrap().into(),
        ];
        let native_result = stages::run(runner.bin(), args, stages::Stage::Slice, &directory);
        let native_failure = native_result.expect_err(&format!(
            "{directory:?}: native must crash on `{}`, but exited 0",
            leg.pattern
        ));
        let (native_code, native_status) = native_failure
            .captured
            .as_ref()
            .map(|c| (c.code, c.status.clone()))
            .unwrap_or((None, String::new()));
        let error = public.expect_err("crash-class pattern must remain unsupported");
        assert!(
            format!("{error:?}").contains("ironing_pattern"),
            "{:?}: error must carry `ironing_pattern`, got {error:?}",
            directory
        );
        fs::write(
            directory.join("comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "label": leg.label,
                "pattern": leg.pattern,
                "native_crashes": true,
                "native_failure": format!("{native_failure}"),
                "native_code": native_code,
                "native_status": native_status,
                "ares_error": format!("{error:?}"),
            }))
            .unwrap(),
        )
        .unwrap();
        return;
    }

    let reference = native(&runner, &directory, &project);
    if let Ok(gcode) = &public {
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
    let cli = Command::new(binary)
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
    let raw_equals_cli = match (&public, fs::read(&output)) {
        (Ok(api), Ok(cli)) => api == &cli,
        _ => false,
    };
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label": leg.label,
            "pattern": leg.pattern,
            "input_sha256": sha(&project),
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

macro_rules! legs {
    ($($name:ident => $index:expr,)*) => {$(#[test] fn $name() { run_leg(&LEGS[$index]); })*};
}

legs! {
    orca_parity_ironing_pattern_control_rectilinear => 0,
    orca_parity_ironing_pattern_monotonic => 1,
    orca_parity_ironing_pattern_monotonicline => 2,
    orca_parity_ironing_pattern_alignedrectilinear => 3,
    orca_parity_ironing_pattern_line => 4,
    orca_parity_ironing_pattern_grid => 5,
    orca_parity_ironing_pattern_triangles => 6,
    orca_parity_ironing_pattern_tri_hexagon => 7,
    orca_parity_ironing_pattern_cubic => 8,
    orca_parity_ironing_pattern_quartercubic => 9,
    orca_parity_ironing_pattern_honeycomb => 10,
    orca_parity_ironing_pattern_3dhoneycomb => 11,
    orca_parity_ironing_pattern_lateral_honeycomb => 12,
    orca_parity_ironing_pattern_crosshatch => 13,
    orca_parity_ironing_pattern_tpmsd => 14,
    orca_parity_ironing_pattern_tpmsfk => 15,
    orca_parity_ironing_pattern_gyroid => 16,
    orca_parity_ironing_pattern_adaptivecubic => 17,
    orca_parity_ironing_pattern_supportcubic => 18,
    orca_parity_ironing_pattern_lightning => 19,
}
