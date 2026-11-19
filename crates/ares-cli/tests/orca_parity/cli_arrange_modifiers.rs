#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{
    golden::{GeneratorKind, compare_ordered_bytes_generator_only, normalize_one_generator_line},
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, stages},
};

const ROTATION: &[u8] =
    include_bytes!("../../../../tests/parity/cli-arrange-modifiers/rotation.3mf");
const ALIGNMENT: &[u8] =
    include_bytes!("../../../../tests/parity/cli-arrange-modifiers/alignment.3mf");
const COLORS: &[u8] =
    include_bytes!("../../../../tests/parity/cli-arrange-modifiers/multicolor.3mf");

#[test]
fn rotation_fits_a_part_wider_than_the_bed() {
    let Some(oracle) = Oracle::new("rotation", ROTATION, None) else {
        return;
    };
    let fixed = oracle.native("fixed", &["--align-to-y-axis=0"], 206);
    let flags = &["--align-to-y-axis=0", "--allow-rotations"];
    let rotating = oracle.native("rotating", flags, 0);
    assert!(fixed.gcode.is_none() && rotating.gcode.is_some());
    oracle.compare("fixed", &["--align-to-y-axis=0"], 206, &fixed);
    oracle.compare("rotating", flags, 0, &rotating);
}

#[test]
fn explicit_alignment_overrides_the_i3_default() {
    let Some(oracle) = Oracle::new("alignment", ALIGNMENT, None) else {
        return;
    };
    let default = oracle.native("default", &[], 0);
    let flags_off = &["--align-to-y-axis=OFF"];
    let off = oracle.native("off", flags_off, 0);
    let flags_on = &["--align-to-y-axis=off", "--align-to-y-axis"];
    let on = oracle.native("on", flags_on, 0);
    assert!(default.native_stream() != off.native_stream());
    assert!(default.native_stream() == on.native_stream());
    let default = oracle.compare("default", &[], 0, &default);
    let off = oracle.compare("off", flags_off, 0, &off);
    let on = oracle.compare("on", flags_on, 0, &on);
    assert!(default.gcode != off.gcode);
    assert!(default.gcode == on.gcode);
}

#[test]
fn disabling_multicolor_rejects_two_material_groups_on_the_selected_plate() {
    let Some(oracle) = Oracle::new("multicolor", COLORS, Some("1")) else {
        return;
    };
    let same_plate = oracle.native("shared", &[], 0);
    let flags = &["--allow-multicolor-oneplate=0"];
    let split = oracle.native("split", flags, 235);
    assert!(same_plate.gcode.is_some() && split.gcode.is_none());
    oracle.compare("shared", &[], 0, &same_plate);
    oracle.compare("split", flags, 235, &split);
}

#[test]
fn calibration_region_flag_leaves_direct_project_arrangement_unchanged() {
    let Some(oracle) = Oracle::new("calibration-region", COLORS, Some("1")) else {
        return;
    };
    let omitted = oracle.native("omitted", &[], 0);
    let flags = &["--avoid-extrusion-cali-region"];
    let enabled = oracle.native("enabled", flags, 0);
    assert!(omitted.native_stream() == enabled.native_stream());
    assert_eq!(omitted.report, enabled.report);
    let omitted = oracle.compare("omitted", &[], 0, &omitted);
    let enabled = oracle.compare("enabled", flags, 0, &enabled);
    assert!(omitted.gcode == enabled.gcode);
    assert_eq!(omitted.report, enabled.report);
}

struct Outputs {
    gcode: Option<Vec<u8>>,
    report: Vec<u8>,
}

impl Outputs {
    fn read(work: &Path, exit: i32) -> Self {
        let stream = fs::read(work.join("plate_1.gcode"));
        if exit != 0 {
            assert_eq!(
                stream.as_ref().unwrap_err().kind(),
                std::io::ErrorKind::NotFound
            );
        }
        Self {
            gcode: (exit == 0).then(|| stream.unwrap()),
            report: result_comparison_bytes(&fs::read(work.join("result.json")).unwrap()),
        }
    }

    fn native_stream(&self) -> Option<Vec<u8>> {
        self.gcode
            .as_ref()
            .map(|bytes| normalize_one_generator_line(bytes, GeneratorKind::Orca).unwrap())
    }
}

struct Oracle {
    runner: OrcaRunner,
    root: std::path::PathBuf,
    input: String,
    plate: Option<&'static str>,
}

impl Oracle {
    fn new(name: &str, fixture: &[u8], plate: Option<&'static str>) -> Option<Self> {
        let runner = OrcaRunner::from_env().unwrap()?;
        let root = tempfile::Builder::new()
            .prefix(&format!("cli-arrange-modifiers-{name}-"))
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let input = root.join("project.3mf");
        fs::write(&input, fixture).unwrap();
        Some(Self {
            runner,
            root,
            input: input.into_os_string().into_string().unwrap(),
            plate,
        })
    }

    fn native(&self, name: &str, flags: &[&str], exit: i32) -> Outputs {
        let run = |kind| {
            let work = self.root.join(name).join(kind);
            fs::create_dir_all(&work).unwrap();
            let args = ["--arrange", "1"]
                .into_iter()
                .chain(flags.iter().copied())
                .chain([
                    "--slice",
                    self.plate.unwrap_or("0"),
                    "--outputdir",
                    ".",
                    &self.input,
                ])
                .map(str::to_owned);
            let result = stages::run_live(self.runner.bin(), args, stages::Stage::Slice, &work);
            assert_eq!(
                result.map_or_else(|err| err.captured.unwrap().code, |()| Some(0)),
                Some(exit),
                "Native: {work:?}"
            );
            Outputs::read(&work, exit)
        };
        let output = run("native");
        let replay = run("native-replay");
        assert_eq!(output.report, replay.report, "Native result replay: {name}");
        assert!(
            output.native_stream() == replay.native_stream(),
            "Native G-code replay: {name}"
        );
        output
    }

    fn compare(&self, name: &str, flags: &[&str], exit: i32, native: &Outputs) -> Outputs {
        let work = self.root.join(name).join("ares");
        fs::create_dir(&work).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
        command
            .current_dir(&work)
            .args(["slice", "-o", "plate_1.gcode", "--arrange", "1"]);
        if let Some(catalog) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") {
            command.arg("--project-preset-catalog").arg(catalog);
        }
        if let Some(plate) = self.plate {
            command.args(["--plate", plate]);
        }
        let result = command.args(flags).arg(&self.input).output().unwrap();
        fs::write(work.join("stdout"), &result.stdout).unwrap();
        fs::write(work.join("stderr"), &result.stderr).unwrap();
        assert_eq!(
            result.status.code(),
            Some(exit),
            "Ares: {work:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = Outputs::read(&work, exit);
        assert_eq!(native.report, actual.report, "result.json: {work:?}");
        if let (Some(expected), Some(actual)) = (&native.gcode, &actual.gcode) {
            compare_ordered_bytes_generator_only(expected, actual)
                .unwrap_or_else(|error| panic!("{work:?}: {error}"));
        }
        actual
    }
}
