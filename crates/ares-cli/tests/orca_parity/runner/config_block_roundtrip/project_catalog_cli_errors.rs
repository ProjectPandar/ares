use super::super::repo_root;

#[test]
fn project_catalog_cli_uptodate_requires_explicit_catalog() {
    let error = rejected(&["--uptodate"]);
    assert!(error.contains("--project-preset-catalog"), "{error}");
}

#[test]
fn project_catalog_cli_replacement_modes_are_exclusive() {
    let error = rejected(&[
        "--project-preset-catalog",
        "missing-catalog.json",
        "--load-settings",
        "machine.json",
    ]);
    assert!(error.contains("cannot be used with"), "{error}");
}

#[test]
fn project_catalog_cli_missing_file_is_an_input_error() {
    let error = rejected(&["--project-preset-catalog", "missing-catalog.json"]);
    assert!(
        error.contains("cannot read project preset catalog"),
        "{error}"
    );
}

fn rejected(extra: &[&str]) -> String {
    let work = tempfile::Builder::new()
        .prefix("project-catalog-cli-errors-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let output = crate::runner::application::cli_command()
        .arg("slice")
        .args(extra)
        .arg("-o")
        .arg(work.join("output.gcode"))
        .arg(repo_root().join("tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"))
        .current_dir(&work)
        .output()
        .unwrap();
    std::fs::write(work.join("stdout"), &output.stdout).unwrap();
    std::fs::write(work.join("stderr"), &output.stderr).unwrap();
    std::fs::write(work.join("status.txt"), output.status.to_string()).unwrap();
    assert!(
        !output.status.success(),
        "invalid CLI request succeeded: {work:?}"
    );
    String::from_utf8(output.stderr).unwrap()
}
