use super::super::{OrcaRunner, repo_root};
use super::project_catalog_supply::{native, produce};

#[test]
fn project_preset_catalog_cli_request_matches_native_applied_config() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let supply = produce(&runner);
    let input = std::fs::read(
        repo_root().join("tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"),
    )
    .unwrap();
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, &input).unwrap();
    let reference = native(&runner, &project, Some(&supply.join("native")), "reference");
    let output = runner.work.join("cli.gcode");
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .arg("slice")
        .arg("--plate")
        .arg("1")
        .arg("--project-preset-catalog")
        .arg(supply.join("catalog.json"))
        .arg("-o")
        .arg(&output)
        .arg(&project)
        .output()
        .unwrap();
    std::fs::write(runner.work.join("cli.stdout"), &cli.stdout).unwrap();
    std::fs::write(runner.work.join("cli.stderr"), &cli.stderr).unwrap();
    std::fs::write(runner.work.join("cli-status.txt"), cli.status.to_string()).unwrap();
    assert!(
        cli.status.success(),
        "public CLI catalog request: {:?}",
        runner.work
    );
    let actual = std::fs::read(output).unwrap();
    let reference = config_block(&reference);
    let actual = config_block(&actual);
    let difference = crate::golden::first_difference(reference.as_bytes(), actual.as_bytes())
        .map(|difference| difference.to_string());
    std::fs::write(
        runner.work.join("config-comparison.txt"),
        format!("{difference:?}"),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "full applied CONFIG_BLOCK: {:?}: {difference:?}",
        runner.work
    );
}

fn config_block(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes)
        .unwrap()
        .split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .split_once("; CONFIG_BLOCK_END")
        .unwrap()
        .0
}
