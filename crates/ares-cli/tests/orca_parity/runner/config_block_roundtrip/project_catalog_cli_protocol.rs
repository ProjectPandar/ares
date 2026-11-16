#![cfg(target_os = "linux")]

use super::super::{
    OrcaRunner, application, cache_identity,
    command::{self, Limits},
    profile_entry, repo_root,
    stages::Stage,
};
use super::project_catalog_supply::native;
use serde_json::json;
use std::{fs, time::Duration};

#[test]
fn supplied_cli_adapter_preserves_filament_only_replacement() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = crate::load_settings_3mf::load_settings_3mf_fixture("compatible-input.3mf");
    let filament =
        crate::load_settings_3mf::load_settings_3mf_fixture("compatible-filament-0.json");
    let input = fs::read(&project).unwrap();
    fs::write(runner.work.join("input.3mf"), &input).unwrap();
    let reference_dir = runner.work.join("native");
    fs::create_dir(&reference_dir).unwrap();
    command::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--load-filaments".to_owned(),
            filament.to_string_lossy().into_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            reference_dir.to_string_lossy().into_owned(),
            project.to_string_lossy().into_owned(),
        ],
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Slice,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap();
    let reference = fs::read(reference_dir.join("plate_1.gcode")).unwrap();
    for assignment in [false, true] {
        let output = runner.work.join(format!("ares-{assignment}.gcode"));
        let mut cli = application::cli_command();
        cli.arg("slice");
        if assignment {
            cli.arg(format!("--load-filaments={}", filament.display()));
        } else {
            cli.arg("--load-filaments").arg(&filament);
        }
        let result = cli.arg("-o").arg(&output).arg(&project).output().unwrap();
        fs::write(
            runner.work.join(format!("cli-{assignment}.stderr")),
            &result.stderr,
        )
        .unwrap();
        fs::write(
            runner.work.join(format!("cli-{assignment}.stdout")),
            &result.stdout,
        )
        .unwrap();
        profile_entry::write_json(
            &runner.work.join(format!("result-{assignment}.json")),
            &json!({
                "input_sha256": cache_identity::raw_digest(&input),
                "filament_sha256": cache_identity::raw_digest(&fs::read(&filament).unwrap()),
                "catalog_supply_present": std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG").is_some(),
                "assignment": assignment, "cli_exit": result.status.code(),
            }),
        );
        assert!(
            result.status.success(),
            "explicit filament-only request: {}; {:?}",
            String::from_utf8_lossy(&result.stderr),
            runner.work
        );
        assert_eq!(
            crate::load_settings_cli::banner_normalized(&reference),
            crate::load_settings_cli::banner_normalized(&fs::read(&output).unwrap()),
            "unchanged filament-only whole-output contract; {:?}",
            runner.work
        );
    }
}

#[test]
fn supplied_cli_adapter_keeps_missing_catalog_uptodate_error() {
    let work = tempfile::Builder::new()
        .prefix("cli-uptodate-protocol-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let output = work.join("output.gcode");
    let result = application::cli_command()
        .args(["slice", "--uptodate", "-o"])
        .arg(&output)
        .arg(repo_root().join("tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"))
        .output()
        .unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    profile_entry::write_json(
        &work.join("result.json"),
        &json!({
            "catalog_supply_present": std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG").is_some(),
            "cli_exit": result.status.code(), "output_created": output.exists(),
        }),
    );
    assert_eq!(
        result.status.code(),
        Some(2),
        "authored missing-catalog request must remain invalid; {work:?}"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("required arguments") && stderr.contains("--project-preset-catalog"),
        "{stderr}; {work:?}"
    );
    assert!(result.stdout.is_empty());
    assert!(!output.exists());
}

#[test]
fn supplied_cli_adapter_still_refreshes_ordinary_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        repo_root().join("tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf");
    let supply = application::project_supply().unwrap();
    let reference = native(
        &runner,
        &project,
        supply.map(|value| value.native_dir.as_path()),
        "native",
    );
    let output = runner.work.join("ares.gcode");
    let result = application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(runner.work.join("stderr"), &result.stderr).unwrap();
    fs::write(runner.work.join("stdout"), &result.stdout).unwrap();
    assert!(
        result.status.success(),
        "ordinary project request; {:?}",
        runner.work
    );
    let actual = fs::read(&output).unwrap();
    let reference = std::str::from_utf8(&reference)
        .unwrap()
        .split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .split_once("; CONFIG_BLOCK_END")
        .unwrap()
        .0;
    let actual = std::str::from_utf8(&actual)
        .unwrap()
        .split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .split_once("; CONFIG_BLOCK_END")
        .unwrap()
        .0;
    let difference = crate::golden::first_difference(reference.as_bytes(), actual.as_bytes())
        .map(|value| value.to_string());
    fs::write(
        runner.work.join("config-comparison.txt"),
        format!("{difference:?}"),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "ordinary automatic supply must remain active: {difference:?}; {:?}",
        runner.work
    );
}
