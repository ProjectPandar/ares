//! APA-R2 M: fixed admitted Arachne geometry with only PA activation/model edits.

use crate::{OrcaRunner, runner};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn adaptive_pa_m_arachne_active_multipath() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let original = include_bytes!(
        "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf"
    );
    let original_sha = Sha256::digest(original)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        original_sha,
        "6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d"
    );
    let edits = [
        ("enable_pressure_advance", json!(["1"])),
        ("adaptive_pressure_advance", json!(["1"])),
        (
            "adaptive_pressure_advance_model",
            json!([
                "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000"
            ]),
        ),
    ];
    let mut before = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..before.len() {
        let mut file = before.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in &edits {
                settings[key] = value.clone();
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let project = writer.finish().unwrap().into_inner();
    let mut original_settings = runner::application::settings(original).unwrap();
    let mut actual_settings = runner::application::settings(&project).unwrap();
    for (key, value) in edits {
        assert_eq!(actual_settings.remove(key), Some(value));
        original_settings.remove(key);
    }
    assert_eq!(
        actual_settings, original_settings,
        "all non-PA settings frozen"
    );
    let mut after = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    assert_eq!(before.len(), after.len());
    for index in 0..before.len() {
        let mut file = before.by_index(index).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut same = after.by_name(file.name()).unwrap();
        let mut a = Vec::new();
        let mut b = Vec::new();
        file.read_to_end(&mut a).unwrap();
        same.read_to_end(&mut b).unwrap();
        assert_eq!(a, b, "unchanged ZIP member {}", file.name());
    }
    let directory = tempfile::Builder::new()
        .prefix("adaptive-pa-m-active-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, &project).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&runner::ExportedCase {
            label: "adaptive-pa/r2-m/arachne-active-multipath".into(),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&case);
    let native = std::str::from_utf8(&case.reference).unwrap();
    let commands: Vec<_> = native
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("M900 K"))
        .collect();
    fs::write(
        directory.join("apa-r2-m-active.json"),
        serde_json::to_vec_pretty(&json!({
            "original_sha256": original_sha,
            "status": outcome.status, "detail": outcome.detail,
            "artifacts": outcome.artifacts, "native_pa_commands": commands,
            "source": "GCode.cpp:6038-6066,6722-6742,6816-6841; AdaptivePAProcessor.cpp:214-295",
            "scope": "M900 admitted by owner; complete-output comparison, not PA-only comparison; multipath runtime reachability still needs source trace"
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        commands
            .iter()
            .map(|(_, line)| line)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 1,
        "Native adaptive PA did not change the static PA"
    );
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
}
