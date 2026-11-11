//! `--progress`: every `PrintBase::set_status` status of the slice on stderr, in the text of
//! upstream's default output `Percent %1%: %2%` (`PrintBase.cpp:127`).
//! Failure inventory: a status missing, duplicated or out of order; another format; the lines on
//! stdout; other stderr output changed; the G-code changed by the flag.

use std::{fs, path::Path, process::Output};

fn run(input: &Path, output: &Path, progress: bool) -> Output {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ares");
    command.arg("slice");
    if progress {
        command.arg("--progress");
    }
    let run = command.arg("-o").arg(output).arg(input).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    run
}

fn core_statuses(project: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut record = |status: &ares_core::SlicingStatus<'_>| {
        lines.push(format!("Percent {}: {}", status.percent, status.text));
    };
    let mut context = ares_core::SlicerContext::new(0);
    context.set_status_callback(&mut record);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime
        .block_on(ares_core::slice_project_plate(
            project,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            context,
        ))
        .unwrap();
    lines
}

#[test]
fn progress_prints_each_slicing_status_to_stderr_and_nothing_else_changes() {
    let directory = tempfile::tempdir().unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/organic_merge_affinity/overhang-organic.3mf");
    let quiet_output = directory.path().join("quiet.gcode");
    let progress_output = directory.path().join("progress.gcode");
    let quiet = run(&input, &quiet_output, false);
    let progress = run(&input, &progress_output, true);

    assert_eq!(
        fs::read(&progress_output).unwrap(),
        fs::read(&quiet_output).unwrap()
    );
    assert_eq!(progress.stdout, quiet.stdout);
    let stderr = String::from_utf8(progress.stderr).unwrap();
    let (statuses, rest): (Vec<&str>, Vec<&str>) = stderr
        .lines()
        .partition(|line| line.starts_with("Percent "));
    assert_eq!(
        rest,
        String::from_utf8(quiet.stderr)
            .unwrap()
            .lines()
            .collect::<Vec<_>>()
    );
    let expected = core_statuses(&fs::read(&input).unwrap());
    assert!(expected.contains(&"Percent 80: Generating G-code: layer 1".to_owned()));
    assert_eq!(statuses, expected);
}
