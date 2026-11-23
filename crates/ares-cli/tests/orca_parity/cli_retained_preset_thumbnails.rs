#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Write},
    os::unix::fs::symlink,
    path::Path,
    process::Command,
};

use serde_json::json;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::runner::{OrcaRunner, application::settings, repo_root};

#[test]
fn clean_project_keeps_the_loaded_thumbnail_string() {
    compare(
        ["", "", ""],
        "48x48/PNG,300x300/PNG",
        "48x48/PNG, 300x300/PNG",
        false,
    );
}

#[test]
fn dirty_process_uses_the_retained_printer_thumbnail_string() {
    compare(
        ["enable_prime_tower", "", ""],
        "32x32/PNG,64x64/PNG",
        "48x48/PNG,300x300/PNG",
        false,
    );
}

#[test]
fn dirty_printer_thumbnail_key_keeps_the_loaded_sizes() {
    compare(
        ["enable_prime_tower", "", "thumbnails"],
        "32x32/PNG,64x64/PNG",
        "32x32/PNG, 64x64/PNG",
        false,
    );
}

#[test]
fn dirty_process_uses_the_installed_printer_thumbnail_string() {
    compare(
        ["enable_prime_tower", "", ""],
        "48x48/PNG,300x300/PNG",
        "48x48/PNG,300x300/PNG",
        true,
    );
}

fn compare(dirty: [&str; 3], thumbnails: &str, expected: &str, use_installed_resources: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let repo = repo_root();
    let resources = repo.join("OrcaSlicer/resources");
    let input = fs::read(repo.join("tests/parity/adaptive-cubic-bodies/x1c-stock.3mf")).unwrap();
    let project = root.join("one-filament.3mf");
    write_project(&project, &input, dirty, thumbnails);

    let expected_lines = [
        format!("; thumbnails = {expected}"),
        "; enable_prime_tower = 0".into(),
    ];
    let fields = |actual: &str| {
        ["; thumbnails = ", "; enable_prime_tower = "].map(|prefix| {
            actual
                .lines()
                .find(|line| line.starts_with(prefix))
                .unwrap()
                .to_owned()
        })
    };
    let mut native = Command::new(runner.bin());
    native.env_remove("DISPLAY").env_remove("WAYLAND_DISPLAY");
    let native = fields(&slice(native, (&project, &root.join("native"))));
    assert_eq!(native, expected_lines, "Native project at {root:?}");

    let binary = Path::new(env!("CARGO_BIN_EXE_ares"));
    let ares = if use_installed_resources {
        let install = root.join("install");
        let bin = install.join("bin");
        fs::create_dir_all(&bin).unwrap();
        symlink(&resources, install.join("resources")).unwrap();
        let installed = bin.join("ares");
        fs::copy(binary, &installed).unwrap();
        Command::new(installed)
    } else {
        let mut command = Command::new(binary);
        command.arg("slice").arg("--resources-dir").arg(resources);
        command
    };
    let actual = fields(&slice(ares, (&project, &root.join("ares"))));
    assert_eq!(actual, native, "Ares project at {root:?}");
}

fn slice(mut command: Command, (input, out): (&Path, &Path)) -> String {
    fs::create_dir(out).unwrap();
    let output = command
        .current_dir(out)
        .args(["--slice", "1", "--outputdir"])
        .arg(out)
        .arg(input)
        .output()
        .unwrap();
    fs::write(out.join("stdout"), &output.stdout).unwrap();
    fs::write(out.join("stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{command:?} failed in {out:?}");
    fs::read_to_string(out.join("plate_1.gcode")).unwrap()
}

fn write_project(path: &Path, input: &[u8], dirty: [&str; 3], thumbnails: &str) {
    let mut config = settings(input).unwrap();
    for (key, value) in [
        ("different_settings_to_system", json!(dirty)),
        ("enable_prime_tower", json!("0")),
        ("thumbnails", json!(thumbnails)),
        ("thumbnails_format", json!("PNG")),
    ] {
        config.insert(key.to_owned(), value);
    }
    let mut source = ZipArchive::new(Cursor::new(input)).unwrap();
    let mut project = ZipWriter::new(fs::File::create(path).unwrap());
    for index in 0..source.len() {
        let file = source.by_index(index).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            project
                .start_file(file.name(), SimpleFileOptions::default())
                .unwrap();
            project
                .write_all(&serde_json::to_vec(&config).unwrap())
                .unwrap();
        } else {
            project.raw_copy_file(file).unwrap();
        }
    }
    project.finish().unwrap();
}
