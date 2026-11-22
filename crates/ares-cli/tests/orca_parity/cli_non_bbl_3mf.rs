#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{
    artifacts,
    load_settings_cli::{banner_normalized, preset_files},
    native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_non_bbl_3mf_omits_upward_compatible_machine_line() {
    let Some(fixture) = Fixture::new("distinct") else {
        return;
    };
    for project in [Project::Bbl, Project::Plain] {
        let native = fixture.slice(Engine::Native, project, "a.json;b.json");
        fixture
            .slice(Engine::Ares, project, "a.json;b.json")
            .compare(&native, &fixture.root, project.name());
    }
}

#[test]
fn cli_non_bbl_3mf_keeps_tower_for_one_loaded_preset() {
    let Some(fixture) = Fixture::new("one-preset") else {
        return;
    };
    for project in [Project::Bbl, Project::Plain] {
        let native = fixture.slice(Engine::Native, project, "a.json;a.json");
        fixture
            .slice(Engine::Ares, project, "a.json;a.json")
            .compare(&native, &fixture.root, project.name());
    }
}

#[derive(Clone, Copy)]
enum Engine {
    Native,
    Ares,
}

/// The Native-exported project, and its copy without the generator tags that set
/// `m_is_bbl_3mf` (`bbs_3mf.cpp:4170-4186`).
#[derive(Clone, Copy)]
enum Project {
    Bbl,
    Plain,
}

impl Project {
    fn name(self) -> &'static str {
        match self {
            Self::Bbl => "bbl",
            Self::Plain => "plain",
        }
    }
}

struct Fixture {
    root: PathBuf,
    runner: OrcaRunner,
}

impl Fixture {
    /// Two cubes on filaments 1 and 2 of the Creality Ender-3 0.4 presets with the prime tower on.
    fn new(name: &str) -> Option<Self> {
        let runner = OrcaRunner::from_env().unwrap()?;
        let root = tempfile::Builder::new()
            .prefix(&format!("cli-non-bbl-3mf-{name}-"))
            .tempdir_in(artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
        let mut process: Value = serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
        process["enable_prime_tower"] = "1".into();
        fs::write(&presets[1], serde_json::to_vec(&process).unwrap()).unwrap();
        let mut filament: Value = serde_json::from_slice(&fs::read(&presets[2]).unwrap()).unwrap();
        filament["name"] = "Synthetic filament A".into();
        filament["filament_id"] = "CLI_A".into();
        fs::write(root.join("a.json"), serde_json::to_vec(&filament).unwrap()).unwrap();
        filament["name"] = "Synthetic filament B".into();
        filament["filament_id"] = "CLI_B".into();
        filament["nozzle_temperature"] = serde_json::json!(["220"]);
        fs::write(root.join("b.json"), serde_json::to_vec(&filament).unwrap()).unwrap();
        let stl = root.join("cube10.stl");
        fs::copy(
            crate::runner::repo_root().join("tests/parity/cube10.stl"),
            &stl,
        )
        .unwrap();
        let fixture = Self { root, runner };
        let export = fixture.root.join("export");
        fs::create_dir_all(&export).unwrap();
        let result = Command::new(fixture.runner.bin())
            .current_dir(&export)
            .args(["--outputdir", ".", "--arrange", "1"])
            .args(fixture.load_args("a.json;a.json"))
            .args(["--load-filament-ids", "1,2", "--export-3mf", "bbl.3mf"])
            .args([&stl, &stl])
            .output()
            .unwrap();
        assert!(result.status.success(), "Native export at {export:?}");
        let bbl = fs::read(export.join("bbl.3mf")).unwrap();
        fs::write(fixture.project(Project::Bbl), &bbl).unwrap();
        fs::write(
            fixture.project(Project::Plain),
            without_generator_tags(&bbl),
        )
        .unwrap();
        Some(fixture)
    }

    fn project(&self, project: Project) -> PathBuf {
        self.root.join(format!("{}.3mf", project.name()))
    }

    fn load_args(&self, filaments: &str) -> [String; 4] {
        let path = |name: &str| self.root.join(name).to_string_lossy().into_owned();
        [
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), path("process.json")),
            "--load-filaments".to_owned(),
            filaments.split(';').map(path).collect::<Vec<_>>().join(";"),
        ]
    }

    fn command(&self, engine: Engine, label: &str) -> (Command, PathBuf) {
        let (bin, engine) = match engine {
            Engine::Native => (self.runner.bin().as_path(), "native"),
            Engine::Ares => (Path::new(env!("CARGO_BIN_EXE_ares")), "ares"),
        };
        let cwd = self.root.join(format!("{engine}-{label}"));
        fs::create_dir_all(cwd.join("out")).unwrap();
        let mut command = Command::new(bin);
        command.current_dir(&cwd);
        (command, cwd)
    }

    fn slice(&self, engine: Engine, project: Project, filaments: &str) -> SliceOutput {
        let (mut command, cwd) = self.command(engine, project.name());
        let result = command
            .args(["--slice", "1", "--arrange", "0", "--outputdir", "out"])
            .args(self.load_args(filaments))
            .arg(self.project(project))
            .output()
            .unwrap();
        fs::write(cwd.join("exit"), format!("{:?}\n", result.status.code())).unwrap();
        fs::write(cwd.join("stderr"), &result.stderr).unwrap();
        fs::write(cwd.join("stdout"), &result.stdout).unwrap();
        assert_eq!(result.status.code(), Some(0), "{cwd:?}");
        let out = cwd.join("out");
        let mut files = fs::read_dir(&out)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        files.sort_unstable();
        assert_eq!(files, ["plate_1.gcode", "result.json"], "{cwd:?}");
        SliceOutput {
            stderr: result.stderr,
            report: result_comparison_bytes(&fs::read(out.join("result.json")).unwrap()),
            gcode: banner_normalized(&fs::read(out.join("plate_1.gcode")).unwrap()),
        }
    }
}

/// Replaces the `Application` metadata with another slicer's and drops the `OrcaSlicer` version
/// metadata, so the importer reads the project as a non-BBL 3MF.
pub(super) fn without_generator_tags(project: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let application =
        regex::Regex::new(r#"<metadata name="Application">[^<]*</metadata>"#).unwrap();
    let version = regex::Regex::new(r#"\s*<metadata name="OrcaSlicer">[^<]*</metadata>"#).unwrap();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/3dmodel.model" {
            let model = String::from_utf8(bytes).unwrap();
            let renamed = application.replace(
                &model,
                r#"<metadata name="Application">PrusaSlicer-2.8.1</metadata>"#,
            );
            let plain = version.replace(&renamed, "").into_owned();
            assert!(
                !plain.contains("OrcaSlicer-")
                    && !plain.contains("BambuStudio-")
                    && !plain.contains(r#"name="OrcaSlicer""#)
                    && plain.contains("PrusaSlicer-2.8.1"),
                "generator tags remain"
            );
            bytes = plain.into_bytes();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

struct SliceOutput {
    stderr: Vec<u8>,
    report: Vec<u8>,
    gcode: Vec<u8>,
}

impl SliceOutput {
    fn compare(&self, expected: &Self, root: &Path, label: &str) {
        assert_eq!(self.stderr, expected.stderr, "stderr at {root:?} {label}");
        assert!(
            self.report == expected.report,
            "result.json at {root:?} {label}"
        );
        assert!(self.gcode == expected.gcode, "G-code at {root:?} {label}");
    }
}
