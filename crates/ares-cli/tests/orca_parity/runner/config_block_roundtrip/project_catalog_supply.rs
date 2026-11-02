use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use std::{path::Path, time::Duration};

#[test]
fn project_catalog_producer_preserves_existing_native_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let supply = produce(&runner);
    let fixture = repo_root().join("tests/parity/crosshatch-ordering/control.3mf");
    let input = std::fs::read(fixture).unwrap();
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, &input).unwrap();
    std::fs::write(
        runner.work.join("input-sha256.txt"),
        cache_identity::raw_digest(&input),
    )
    .unwrap();
    let bare = native(&runner, &project, None, "bare");
    let staged = native(&runner, &project, Some(&supply.join("native")), "staged");
    let bare =
        crate::golden::normalize_one_generator_line(&bare, crate::golden::GeneratorKind::Orca)
            .unwrap();
    let staged =
        crate::golden::normalize_one_generator_line(&staged, crate::golden::GeneratorKind::Orca)
            .unwrap();
    let comparison =
        crate::golden::first_difference(&bare, &staged).map(|difference| difference.to_string());
    std::fs::write(
        runner.work.join("native-control-comparison.txt"),
        format!("{comparison:?}"),
    )
    .unwrap();
    assert!(
        comparison.is_none(),
        "Native installed-preset control: {:?}: {comparison:?}",
        runner.work
    );
}

pub(super) fn produce(runner: &OrcaRunner) -> std::path::PathBuf {
    let supply = runner.work.join("supply");
    let generated = std::process::Command::new("node")
        .arg(repo_root().join("scripts/create-project-preset-catalog.mjs"))
        .arg("--profiles")
        .arg(repo_root().join("OrcaSlicer/resources/profiles"))
        .arg("--out")
        .arg(&supply)
        .output()
        .unwrap();
    std::fs::write(runner.work.join("producer.stdout"), &generated.stdout).unwrap();
    std::fs::write(runner.work.join("producer.stderr"), &generated.stderr).unwrap();
    assert!(
        generated.status.success(),
        "public catalog producer: {:?}",
        runner.work
    );
    let catalog: serde_json::Value =
        serde_json::from_slice(&std::fs::read(supply.join("catalog.json")).unwrap()).unwrap();
    assert!(catalog["vendors"].as_array().unwrap().len() > 10);
    supply
}

pub(super) fn native(
    runner: &OrcaRunner,
    project: &Path,
    datadir: Option<&Path>,
    label: &str,
) -> Vec<u8> {
    let output = runner.work.join(label);
    std::fs::create_dir(&output).unwrap();
    let mut args = vec![
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_string_lossy().into_owned(),
    ];
    if let Some(datadir) = datadir {
        args.extend([
            "--datadir".to_owned(),
            datadir.to_string_lossy().into_owned(),
        ]);
    }
    args.push(project.to_string_lossy().into_owned());
    command::run_exact(
        runner.bin(),
        args,
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Slice,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap_or_else(|error| panic!("Native {label}: {error}; {:?}", runner.work));
    std::fs::read(output.join("plate_1.gcode")).unwrap()
}
