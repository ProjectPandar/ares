//! Ares-defined behavior, not a Native parity oracle: see
//! `docs/architecture/support-ironing-fill-params-beta.md`.

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Cursor, path::PathBuf};

const INPUT: &[u8] =
    include_bytes!("../../../tests/defined-behavior/support-ironing/three-d-honeycomb.3mf");

#[test]
fn support_ironing_three_d_honeycomb_nonfinite_z_scale_has_no_ironing_paths_or_extrusion() {
    let input_sha: String = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        input_sha,
        "195d59c8bc339fa72d2b7959575185c3cbfa0bd3ce3a9c6b1a3562041f73ad1e"
    );
    let mut archive = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let config: serde_json::Value =
        serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap())
            .unwrap();
    assert_eq!(config["support_ironing"], "1");
    assert_eq!(
        config["support_ironing_pattern"],
        serde_json::to_value(ares_core::ProcessInfillPattern::ThreeDHoneycomb).unwrap()
    );
    assert_eq!(config["ironing_type"], "no ironing");
    assert_eq!(config["prime_tower_flat_ironing"], "0");

    let artifact_root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&artifact_root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix("ares-defined-support-ironing-3dh-")
        .tempdir_in(artifact_root)
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), INPUT).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut outputs = Vec::with_capacity(2);
    for replay in 0..2 {
        let gcode = runtime
            .block_on(ares_core::slice_project_plate_validated(
                INPUT,
                GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ConfigOverrides::default(),
                SlicerContext::new(0),
            ))
            .unwrap();
        fs::write(directory.join(format!("ares.api.{replay}.gcode")), &gcode).unwrap();
        outputs.push(gcode);
    }
    let gcode = std::str::from_utf8(&outputs[0]).unwrap();
    assert!(gcode.contains(";TYPE:Support\n"), "{directory:?}");
    assert!(gcode.contains(";TYPE:Support interface\n"), "{directory:?}");

    let mut ironing = false;
    let mut ironing_role_blocks = 0;
    let mut ironing_motion_commands = 0;
    let mut positive_e_commands = 0;
    for line in gcode.lines() {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            ironing = role == "Ironing";
            ironing_role_blocks += usize::from(ironing);
        }
        if !ironing {
            continue;
        }
        let mut words = line.split_whitespace();
        if !matches!(words.next(), Some("G0" | "G1" | "G2" | "G3")) {
            continue;
        }
        ironing_motion_commands += 1;
        for word in words {
            if let Some(e) = word.strip_prefix('E')
                && e.parse::<f64>().unwrap() > 0.0
            {
                positive_e_commands += 1;
            }
        }
    }
    fs::write(
        directory.join("observation.json"),
        serde_json::to_vec_pretty(&json!({
            "contract": "ARES_DEFINED_NONFINITE_Z_SCALE_EMPTY_COMPONENT",
            "ard": "docs/architecture/support-ironing-fill-params-beta.md",
            "native_comparison": false,
            "input_sha256": input_sha,
            "public_api_replays_byte_equal": outputs[0] == outputs[1],
            "ironing_role_blocks": ironing_role_blocks,
            "ironing_motion_commands": ironing_motion_commands,
            "positive_e_commands": positive_e_commands,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outputs[0], outputs[1], "API replay: {directory:?}");
    assert_eq!(
        (
            ironing_role_blocks,
            ironing_motion_commands,
            positive_e_commands
        ),
        (0, 0, 0),
        "Ares-defined empty support-ironing component: {directory:?}"
    );
}
