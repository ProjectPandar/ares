use std::{
    ffi::OsString,
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::Output,
};

use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::super::{cli_no_check::run, export_3mf_sliced::write_saved_input};
use crate::OrcaRunner;

use super::{Outcome, Saved};

const RELATIVE: &str = "inputs/sub/.././project.3mf";
const KEY: &str = "support_ironing_pattern";
const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const MULTI: &[u8] =
    include_bytes!("../../../../../tests/parity/custom-gcode-overview/two-plates.3mf");
const INVALID: &[u8] =
    include_bytes!("../../../../../tests/parity/spiral-vase-admission/two-object-by-layer.3mf");

pub(super) fn check(label: &str, saved: Saved, native: &[&str], ares: &[&str], outcome: Outcome) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let input = project(&runner, root, saved);
    let absolute = label == "direct-absolute";
    let cwd = root.join("cwd");
    let spelling = if absolute {
        format!("{}/{RELATIVE}", cwd.display())
    } else {
        RELATIVE.to_owned()
    };
    let file = if absolute {
        spelling.clone()
    } else {
        format!("{}/inputs/project.3mf", cwd.display())
    };
    let report = if native.contains(&"--outputdir") {
        "out/result.json"
    } else {
        "result.json"
    };
    let expect_warning = !matches!(saved, Saved::Newer | Saved::Legacy);
    let expected_stderr = format!(
        r#"warning: {file}: value "cross-zag" for {KEY} is not valid, using "rectilinear" instead"#
    );
    let expected_records = if expect_warning {
        json!([{"class": "config_value_substituted", "file": file, "key": KEY,
                "old_value": "cross-zag", "new_value": "rectilinear"}])
    } else {
        json!([])
    };
    for (engine, bin, args) in [
        ("native", runner.bin().as_path(), native),
        ("ares", Path::new(env!("CARGO_BIN_EXE_ares")), ares),
    ] {
        let argv: Vec<_> = args
            .iter()
            .map(OsString::from)
            .chain([spelling.clone().into()])
            .collect();
        fs::create_dir_all(cwd.join("inputs/sub")).unwrap();
        fs::create_dir_all(cwd.join("exports")).unwrap();
        if matches!(outcome, Outcome::BlockedArchive) {
            fs::create_dir_all(cwd.join("exports/project.3mf/existing")).unwrap();
        }
        fs::write(cwd.join("inputs/project.3mf"), &input).unwrap();
        fs::write(
            root.join(format!("{engine}.argv.json")),
            serde_json::to_vec(&argv).unwrap(),
        )
        .unwrap();
        let output = run(bin, root, engine, &argv);
        let saved_cwd = root.join(engine).join("cwd");
        check_outcome(outcome, engine, &saved_cwd, &output);
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        let substitutions: Vec<_> = stderr
            .lines()
            .filter(|line| line.contains(&format!(" for {KEY} is not valid,")))
            .collect();
        let expected: Vec<_> = expect_warning
            .then_some(expected_stderr.as_str())
            .into_iter()
            .collect();
        assert_eq!(
            substitutions, expected,
            "{label}: {engine} load stderr in {root:?}"
        );
        if matches!(outcome, Outcome::MissingPreset) && engine == "ares" {
            assert!(
                stderr.starts_with(&expected_stderr) && stderr.contains("missing-machine.json"),
                "{label}: {stderr}"
            );
        }
        let path = saved_cwd.join(report);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("{label}: missing report at {path:?}: {error}"));
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            result["warnings"], expected_records,
            "{label}: load warning records in {path:?}"
        );
    }
}

fn check_outcome(outcome: Outcome, engine: &str, cwd: &Path, output: &Output) {
    let code = match outcome {
        Outcome::Fail(code) => code,
        Outcome::BlockedArchive => 243,
        Outcome::MissingPreset if engine == "native" => 253,
        Outcome::MissingPreset => 1,
        _ => 0,
    };
    assert_eq!(
        output.status.code(),
        Some(code),
        "{cwd:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = match outcome {
        Outcome::Gcode(names) => names,
        _ => &[],
    };
    let mut gcode: Vec<_> = fs::read_dir(cwd.join("out"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".gcode"))
        .collect();
    gcode.sort_unstable();
    assert_eq!(gcode, expected, "{cwd:?}");
    assert_eq!(
        cwd.join("exports/project.3mf").is_file(),
        matches!(outcome, Outcome::Archive)
    );
    let settings = cwd.join("exports/settings.json");
    assert_eq!(settings.is_file(), matches!(outcome, Outcome::SettingsFile));
    let config = match outcome {
        Outcome::SettingsFile => fs::read(settings).unwrap(),
        Outcome::SettingsStdout => output.stdout.clone(),
        _ => return,
    };
    let config: Value = serde_json::from_slice(&config).unwrap();
    assert_eq!(config[KEY], "rectilinear", "{cwd:?}");
}

fn project(runner: &OrcaRunner, root: &Path, saved: Saved) -> Vec<u8> {
    let generated;
    let input = match saved {
        Saved::Multi | Saved::NonprintableSecond => MULTI,
        Saved::Invalid => INVALID,
        Saved::One | Saved::Newer | Saved::Legacy => {
            let source = root.join("source/cwd");
            fs::create_dir_all(&source).unwrap();
            write_saved_input(runner, &source);
            generated = fs::read(source.join("saved.3mf")).unwrap();
            &generated
        }
    };
    let mut archive = ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).unwrap();
        let name = member.name().to_owned();
        if name == PROJECT_SETTINGS {
            let mut config: Value = serde_json::from_reader(&mut member).unwrap();
            config[KEY] = json!(if matches!(saved, Saved::Legacy) {
                "zig-zag"
            } else {
                "cross-zag"
            });
            output
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            serde_json::to_writer(&mut output, &config).unwrap();
        } else if name.ends_with(".model")
            && matches!(saved, Saved::NonprintableSecond | Saved::Newer)
        {
            let mut xml = String::new();
            member.read_to_string(&mut xml).unwrap();
            let (old, new) = if matches!(saved, Saved::Newer) {
                (
                    r#"<metadata name="OrcaSlicer">2.5.0</metadata>"#,
                    r#"<metadata name="OrcaSlicer">3.5.0</metadata>"#,
                )
            } else {
                (
                    r#" 374 110 5" printable="1""#,
                    r#" 374 110 5" printable="0""#,
                )
            };
            output
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            output.write_all(xml.replace(old, new).as_bytes()).unwrap();
        } else {
            output.raw_copy_file(member).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}
