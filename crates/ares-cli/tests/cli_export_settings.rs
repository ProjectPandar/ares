use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

#[test]
fn exports_complete_project_settings_to_a_file_without_stdout() {
    compare("settings.json", false);
}

#[test]
fn exports_complete_project_settings_as_the_only_stdout_document() {
    compare("-", false);
}

#[test]
fn exports_normalized_spiral_vase_settings() {
    compare("-", true);
}

fn compare(destination: &str, spiral: bool) {
    let Some(binary) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let artifacts = native_environment::value("ARES_PARITY_ARTIFACT_ROOT").unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let root = tempfile::Builder::new()
        .prefix("export-settings-")
        .tempdir_in(artifacts)
        .unwrap()
        .keep();
    eprintln!("artifacts: {}", root.display());
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/orca_cli_ender3.3mf");
    let input = if spiral {
        let path = root.join("spiral.3mf");
        enable_spiral_vase(&input, &path);
        path
    } else {
        input
    };
    let native_dir = root.join("native");
    let ares_dir = root.join("ares");
    fs::create_dir(&native_dir).unwrap();
    fs::create_dir(&ares_dir).unwrap();
    let native = Command::new(binary)
        .current_dir(&native_dir)
        .env("ORCA_250_JOB_APPDIR", native_dir.join("app"))
        .env("ORCA_JOB_APPDIR", native_dir.join("app"))
        .arg("--datadir")
        .arg(native_dir.join("data"))
        .args(["--export-settings", destination])
        .arg(&input)
        .output()
        .unwrap();
    record(&native_dir, &native);
    assert!(native.status.success(), "Native did not export settings");
    let expected = document(destination, &native_dir, &native);
    let actual = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(&ares_dir)
        .args(["--export-settings", destination])
        .arg(&input)
        .output()
        .unwrap();
    record(&ares_dir, &actual);

    assert_eq!(actual.status.code(), native.status.code(), "exit status");
    assert_eq!(actual.stderr, native.stderr, "stderr");
    if destination != "-" {
        assert_eq!(actual.stdout, native.stdout, "stdout");
    }
    let actual = document(destination, &ares_dir, &actual);
    let diff = expected
        .keys()
        .chain(actual.keys())
        .filter(|key| expected.get(*key) != actual.get(*key))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        diff.is_empty(),
        "settings differ at {} keys: {:?}",
        diff.len(),
        diff.iter().take(25).collect::<Vec<_>>()
    );
    assert_eq!(
        fs::read_dir(ares_dir).unwrap().count(),
        if destination == "-" { 3 } else { 4 }
    );
}

fn enable_spiral_vase(input: &Path, output: &Path) {
    use std::io::{Read, Write};

    let mut original = zip::ZipArchive::new(fs::File::open(input).unwrap()).unwrap();
    let mut project = zip::ZipWriter::new(fs::File::create(output).unwrap());
    for index in 0..original.len() {
        let mut member = original.by_index(index).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut bytes = Vec::new();
            member.read_to_end(&mut bytes).unwrap();
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["spiral_mode"] = "1".into();
            settings["wall_loops"] = "3".into();
            settings["top_shell_layers"] = "3".into();
            settings["sparse_infill_density"] = "10%".into();
            settings["resolution"] = "0.0001".into();
            project
                .start_file(member.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            project
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            project.raw_copy_file(member).unwrap();
        }
    }
    project.finish().unwrap();
}

fn document(destination: &str, dir: &Path, output: &Output) -> serde_json::Map<String, Value> {
    let bytes = if destination == "-" {
        output.stdout.clone()
    } else {
        fs::read(dir.join(destination)).unwrap()
    };
    serde_json::from_slice(&bytes).expect("expected one complete settings JSON object")
}

fn record(dir: &Path, output: &Output) {
    fs::write(dir.join("stdout"), &output.stdout).unwrap();
    fs::write(dir.join("stderr"), &output.stderr).unwrap();
    fs::write(dir.join("exit"), format!("{}\n", output.status)).unwrap();
}
