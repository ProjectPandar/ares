//! `support_ironing_pattern` domain handling.
//!
//! Upstream: `PrintConfig.cpp:6565-6573` defines the option as a coEnum with
//! GUI values {rectilinear, concentric} only, but the 3MF project-settings
//! load path does not restrict to that enum domain: any InfillPattern name
//! reaches `SupportParameters.hpp:65-68` and the ironing path generator at
//! `SupportCommon.cpp:1877-1903`, which dispatches through
//! `FillBase.cpp:40-77` (`Fill::new_from_type`) and fills the top contact
//! layer polys with that pattern (role erIroning). `PrintConfig.cpp:8206`
//! additionally normalizes the legacy "zig-zag" alias to rectilinear at
//! config load.
//!
//! Ares currently implements only Rectilinear and Concentric in
//! `support_material/ironing.rs` and rejects every other pattern with
//! `UnsupportedProjectFeature`. This suite pins the Native oracle behavior
//! for patterns ares already implements as surface fills elsewhere
//! (`fill_entities/generate.rs`): grid, monotonic, honeycomb and triangles
//! must slice byte-identically to Native instead of erroring, while the
//! rectilinear and legacy zig-zag legs guard the already-aligned paths.
//!
//! Fixture `tests/parity/support-ironing/overhang-organic.3mf`: organic tree
//! overhang object with `support_ironing=1` and
//! `support_ironing_pattern=rectilinear`; the pattern value is rewritten per
//! leg in `Metadata/project_settings.config`.
use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

pub(super) const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/support-ironing/overhang-organic.3mf");

/// Rewrites `Metadata/project_settings.config`, swapping the
/// `support_ironing_pattern` value. Verifies the fixture shape it patches.
fn project(pattern: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert!(
                text.contains(r#""support_ironing": "1""#),
                "fixture must enable support ironing"
            );
            let needle = r#""support_ironing_pattern": "rectilinear""#;
            assert!(
                text.contains(needle),
                "fixture must carry the rectilinear baseline pattern"
            );
            bytes = text
                .replacen(
                    needle,
                    &format!(r#""support_ironing_pattern": "{pattern}""#),
                    1,
                )
                .into_bytes();
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

/// Rewrites `Metadata/project_settings.config`, swapping the sparse infill
/// pattern value. Control seam for the connect-stage change: sparse
/// lateral-lattice output must stay byte-identical to Native.
fn sparse_project(pattern: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            let needle = r#""sparse_infill_pattern": "crosshatch""#;
            assert!(
                text.contains(needle),
                "fixture must carry the crosshatch sparse baseline"
            );
            bytes = text
                .replacen(
                    needle,
                    &format!(r#""sparse_infill_pattern": "{pattern}""#),
                    1,
                )
                .into_bytes();
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

fn native(runner: &OrcaRunner, directory: &std::path::Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
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
    assert!(!reference.is_empty());
    assert!(
        String::from_utf8_lossy(&reference)
            .matches("ironing")
            .count()
            >= 15,
        "{directory:?}: native oracle must iron the support contact layers"
    );
    reference
}

fn leg(label: &str, pattern: &str) {
    leg_with_project(label, pattern, project(pattern));
}

pub(super) fn leg_with_project(label: &str, pattern: &str, project: Vec<u8>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("support-ironing-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let reference = native(&runner, &directory, &project);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(ares_core::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
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
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"pattern":pattern,
            "input_sha256":sha(&project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

/// Control: rectilinear is the implemented default path and must stay
/// byte-identical to Native before and after the domain slice.
#[test]
fn orca_parity_support_ironing_pattern_rectilinear_control() {
    leg("rect-control", "rectilinear");
}

/// Control: the legacy "zig-zag" alias loads through
/// `PrintConfig.cpp:8206` normalization on the Native side and through the
/// serde alias on the ares side; both must keep matching byte-for-byte.
#[test]
fn orca_parity_support_ironing_pattern_zig_zag_control() {
    leg("zig-zag-control", "zig-zag");
}

/// Domain: grid ironing paths (`FillBase.cpp` ipGrid) must match Native.
#[test]
fn orca_parity_support_ironing_pattern_grid() {
    leg("grid", "grid");
}

/// Domain: monotonic ironing paths (`FillBase.cpp` ipMonotonic).
#[test]
fn orca_parity_support_ironing_pattern_monotonic() {
    leg("monotonic", "monotonic");
}

/// Domain: honeycomb ironing paths (`FillBase.cpp` ipHoneycomb).
#[test]
fn orca_parity_support_ironing_pattern_honeycomb() {
    leg("honeycomb", "honeycomb");
}

/// Domain: triangles ironing paths (`FillBase.cpp` ipTriangles).
#[test]
fn orca_parity_support_ironing_pattern_triangles() {
    leg("triangles", "triangles");
}

/// Domain: gyroid ironing paths (`FillBase.cpp` ipGyroid).
#[test]
fn orca_parity_support_ironing_pattern_gyroid() {
    leg("gyroid", "gyroid");
}

/// Domain: cubic ironing paths (`FillBase.cpp` ipCubic).
#[test]
fn orca_parity_support_ironing_pattern_cubic() {
    leg("cubic", "cubic");
}

/// Domain: quarter-cubic ironing paths (`FillBase.cpp` ipQuarterCubic).
#[test]
fn orca_parity_support_ironing_pattern_quartercubic() {
    leg("quartercubic", "quartercubic");
}

/// Domain: 3D honeycomb ironing paths (`FillBase.cpp` ip3DHoneycomb).
#[test]
fn orca_parity_support_ironing_pattern_three_d_honeycomb() {
    leg("3dhoneycomb", "3dhoneycomb");
}

/// Domain: tri-hexagon ironing paths (`PrintConfig.cpp:238` maps to ipStars).
#[test]
fn orca_parity_support_ironing_pattern_tri_hexagon() {
    leg("tri-hexagon", "tri-hexagon");
}

/// Domain: TPMS-D ironing paths (`FillBase.cpp` ipTpmsD).
#[test]
fn orca_parity_support_ironing_pattern_tpmsd() {
    leg("tpmsd", "tpmsd");
}

/// Domain: TPMS-FK ironing paths (`FillBase.cpp` ipTpmsFK).
#[test]
fn orca_parity_support_ironing_pattern_tpmsfk() {
    leg("tpmsfk", "tpmsfk");
}

/// Domain: lateral honeycomb ironing paths (`FillBase.cpp` ipLateralHoneycomb).
#[test]
fn orca_parity_support_ironing_pattern_lateral_honeycomb() {
    leg("lateral-honeycomb", "lateral-honeycomb");
}

/// Domain: aligned rectilinear ironing paths (`FillBase.cpp`
/// ipAlignedRectilinear shares `fill_surface_by_lines`).
#[test]
fn orca_parity_support_ironing_pattern_aligned_rectilinear() {
    leg("alignedrectilinear", "alignedrectilinear");
}

/// Domain: monotonic line ironing paths (`FillBase.cpp` ipMonotonicLine).
#[test]
fn orca_parity_support_ironing_pattern_monotonic_line() {
    leg("monotonicline", "monotonicline");
}

/// Control: concentric is the pre-slice implemented arm and must stay
/// byte-identical to Native before and after the domain slice.
#[test]
fn orca_parity_support_ironing_pattern_concentric_control() {
    leg("concentric-control", "concentric");
}

/// Domain: lateral lattice ironing paths — `FillLateralLattice` reached
/// from the 3MF load; pins the full-arc degeneracy handling
/// (`FillBase.cpp:474` equal-index walks the whole contour instead of
/// asserting).
#[test]
fn orca_parity_support_ironing_pattern_lateral_lattice() {
    leg("lateral-lattice", "lateral-lattice");
}

/// Control: sparse lateral-lattice (`FillLateralLattice` sparse path
/// through the same `fill/connect` stage) must stay byte-identical to
/// Native while the contour assert is removed.
#[test]
fn orca_parity_sparse_lateral_lattice_control() {
    leg_with_project(
        "sparse-lattice-control",
        "lateral-lattice",
        sparse_project("lateral-lattice"),
    );
}

/// Domain: consistent zig-zag ironing — `FillZigZag` under the canonical
/// enum key (`"zigzag"`, `PrintConfig.cpp:232`; the hyphenated legacy
/// `"zig-zag"` migrates to rectilinear at `PrintConfig.cpp:8206`). The
/// vertical-line grid anchors on the extended dummy support box
/// (`SupportCommon.cpp:1433`/`:1880`); Native emits identical ironing
/// toolpaths for all three consistent patterns, differing only in the
/// config echo (127 E-lines each, probe 2026-10-14).
#[test]
fn orca_parity_support_ironing_pattern_zig_zag() {
    leg("zig-zag", "zigzag");
}

/// Domain: consistent cross-zag ironing — `FillCrossZag`; at the ironing
/// call site every `FillParams` phase/symmetric control keeps its default,
/// matching the zig-zag leg byte-for-byte in Native.
#[test]
fn orca_parity_support_ironing_pattern_cross_zag() {
    leg("cross-zag", "crosszag");
}

/// Domain: consistent locked-zag ironing — `FillLockedZag`; the lock-region
/// extrusion path is unreachable here (only `fill_surface_extrusion` is
/// overridden, the Polylines overload serves the base rectilinear
/// traversal), so this pins the same consistent traversal as zig-zag.
#[test]
fn orca_parity_support_ironing_pattern_locked_zag() {
    leg("locked-zag", "lockedzag");
}

/// Domain: unknown enum keys silently fall to the option default
/// (`Config.cpp:654-672` substitution under the `EnableSilent` 3MF load
/// rule, `Model.cpp:259`) — even the gcode config echo carries the
/// substituted default.
#[test]
fn orca_parity_support_ironing_pattern_unknown_key_cross_zag() {
    leg("unknown-cross-zag", "cross-zag");
}

/// Domain: same substitution for an arbitrary unknown key — Native slices
/// with the default pattern and no warning (probe 2026-10-14:
/// 719 E-lines, echo `rectilinear`).
#[test]
fn orca_parity_support_ironing_pattern_unknown_key_bogus() {
    leg("unknown-bogus", "bogus-pattern");
}
