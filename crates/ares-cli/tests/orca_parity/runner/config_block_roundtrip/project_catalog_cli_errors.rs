use super::super::repo_root;

#[test]
fn project_catalog_cli_uptodate_requires_explicit_catalog() {
    let error = rejected(&["--uptodate"]);
    assert!(error.contains("--project-preset-catalog"), "{error}");
}

#[test]
fn project_catalog_cli_replacement_modes_are_exclusive() {
    for conflicting in ["--load-filaments", "--resources-dir"] {
        let error = rejected(&[
            "--project-preset-catalog",
            "missing-catalog.json",
            conflicting,
            "conflicting-input",
        ]);
        assert!(
            error.contains("cannot be used with")
                && error.contains("--project-preset-catalog")
                && error.contains(conflicting),
            "{error}"
        );
    }
}

#[test]
fn project_catalog_cli_load_settings_reaches_preset_file_loading() {
    let catalog = repo_root().join("tests/profiles/filament-gcode-slots/catalog.json");
    let error = rejected(&[
        "--project-preset-catalog",
        catalog.to_str().unwrap(),
        "--load-settings",
        "missing-machine.json",
    ]);
    assert!(
        error.contains(r#"Loading setting file \"missing-machine.json\" failed:"#),
        "{error}"
    );
    assert!(!error.contains("cannot be used with"), "{error}");
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
