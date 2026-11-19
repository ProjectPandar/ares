//! CLI `--load-settings --progress`, using PrintBase.cpp:127's default status text.
//! Failures: dropped/duplicated/reordered statuses, stdout pollution, changed warnings,
//! changed G-code, or a rejected input mistaken for a successful progress run.

use std::{fs, path::Path, process::Output};

use serde_json::json;
use sha2::{Digest, Sha256};

fn save_output(root: &Path, name: &str, output: &Output) {
    fs::write(root.join(format!("{name}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{name}.stderr")), &output.stderr).unwrap();
    fs::write(
        root.join(format!("{name}.exit")),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn load_settings_progress_preserves_status_order_and_gcode() {
    let root = tempfile::Builder::new()
        .prefix("load-settings-progress-")
        .tempdir_in(
            std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
                .map_or_else(std::env::temp_dir, Into::into),
        )
        .unwrap()
        .keep();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input =
        fs::read(repo.join("tests/parity/master-extruder-id/stock-right-only.3mf")).unwrap();
    let machine =
        fs::read(repo.join("tests/parity/machine-full-resource/stock-machine-full.json")).unwrap();
    fs::write(root.join("input.3mf"), &input).unwrap();
    fs::write(root.join("machine.json"), &machine).unwrap();
    let execute = |name: &str, progress: bool| {
        let mut argv = vec![
            "slice",
            "--load-settings",
            "machine.json",
            "-o",
            name,
            "input.3mf",
        ];
        if progress {
            argv.push("--progress");
        }
        fs::write(
            root.join(format!("{name}.argv.json")),
            serde_json::to_vec_pretty(&argv).unwrap(),
        )
        .unwrap();
        let output = assert_cmd::cargo::cargo_bin_cmd!("ares")
            .args(&argv)
            .current_dir(&root)
            .output()
            .unwrap();
        save_output(&root, name, &output);
        output
    };
    let quiet = execute("quiet.gcode", false);
    let progress = execute("progress.gcode", true);
    let quiet_gcode = fs::read(root.join("quiet.gcode")).ok();
    let progress_gcode = fs::read(root.join("progress.gcode")).ok();
    fs::write(root.join("outputs.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":sha(&input), "machine_sha256":sha(&machine),
        "quiet_exit":quiet.status.code(), "progress_exit":progress.status.code(),
        "quiet_gcode_sha256":quiet_gcode.as_deref().map(sha),
        "progress_gcode_sha256":progress_gcode.as_deref().map(sha),
        "whole_gcode_equal":quiet_gcode.as_ref().is_some_and(|bytes| Some(bytes) == progress_gcode.as_ref())
    })).unwrap()).unwrap();
    assert!(quiet.status.success(), "quiet slice: {root:?}");
    assert!(progress.status.success(), "progress slice: {root:?}");
    assert_eq!(
        progress_gcode.unwrap(),
        quiet_gcode.unwrap(),
        "whole G-code: {root:?}"
    );
    assert_eq!(progress.stdout, quiet.stdout, "stdout: {root:?}");

    let mut expected = Vec::new();
    let mut record = |status: &ares_core::SlicingStatus<'_>| {
        expected.push(format!("Percent {}: {}", status.percent, status.text));
    };
    let mut context = ares_core::SlicerContext::new(0);
    context.set_status_callback(&mut record);
    let settings = [ares_core::LoadSettingsFile {
        label: "machine.json",
        json: std::str::from_utf8(&machine).unwrap(),
    }];
    let overrides = ares_core::ConfigOverrides::default();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_load_settings(
            &input,
            ares_core::LoadSettingsSliceRequest {
                transforms: &ares_core::CliProjectTransforms::NONE,
                filename: "input.3mf",
                catalog_json: None,
                uptodate: false,
                settings: &settings,
                filaments: &[],
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
                sliced_plate: &mut sliced_plate,
            },
            ares_core::GenerationMetadata::default(),
            context,
        ))
        .unwrap();
    assert!(expected.contains(&"Percent 80: Generating G-code: layer 1".to_owned()));
    let stderr = String::from_utf8(progress.stderr).unwrap();
    let (statuses, rest): (Vec<_>, Vec<_>) = stderr
        .lines()
        .partition(|line| line.starts_with("Percent "));
    fs::write(
        root.join("statuses.json"),
        serde_json::to_vec_pretty(&json!({"expected":expected,"actual":statuses})).unwrap(),
    )
    .unwrap();
    eprintln!("load-settings progress artifacts: {}", root.display());
    assert_eq!(statuses, expected, "ordered Print statuses: {root:?}");
    assert_eq!(
        rest,
        String::from_utf8(quiet.stderr)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        "non-status stderr: {root:?}"
    );
}
