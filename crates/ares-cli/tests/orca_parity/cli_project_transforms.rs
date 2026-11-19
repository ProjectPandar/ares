#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{
    golden::{GeneratorKind, compare_ordered_bytes_generator_only, normalize_one_generator_line},
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, stages},
};

#[path = "cli_project_transforms/tilt.rs"]
mod tilt;

const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");

macro_rules! case {
    ($name:ident, $plate:expr, $exit:expr, [$($arg:literal),*]) => {
        #[test]
        fn $name() {
            verify(stringify!($name), $plate, $exit, &[$($arg),*]);
        }
    };
}

case!(untransformed_control, None, 0, []);
case!(forced_arrange, None, 0, ["--arrange", "1"]);
case!(disabled_arrange, None, 0, ["--arrange", "0"]);
case!(automatic_arrange, None, 0, ["--arrange", "-1"]);
case!(uniform_scale, None, 0, ["--scale", "1.2"]);
case!(z_rotation, None, 0, ["--rotate", "45"]);
case!(negative_z_rotation, None, 0, ["--rotate", "-45"]);
case!(
    arrange_after_volume_transforms,
    None,
    0,
    ["--arrange", "1", "--scale", "1.2", "--rotate", "45"]
);
case!(
    selected_plate_arrange_after_volume_transforms,
    Some("1"),
    0,
    ["--arrange", "1", "--scale", "1.2", "--rotate", "45"]
);
case!(zero_scale, None, 254, ["--scale", "0"]);
case!(negative_scale, None, 254, ["--scale", "-1"]);
case!(
    selected_plate_arrange_failure,
    Some("1"),
    235,
    ["--scale", "100", "--arrange", "1"]
);
case!(
    global_arrange_outside_volume,
    None,
    206,
    ["--scale", "100", "--arrange", "1"]
);

fn verify(name: &str, plate: Option<&str>, exit: i32, flags: &[&str]) {
    verify_project(name, PROJECT, plate, exit, flags);
}

fn verify_project(name: &str, project: &[u8], plate: Option<&str>, exit: i32, flags: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-transform-{name}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("project.3mf");
    fs::write(&input, project).unwrap();
    let input = input.to_str().unwrap();
    let mut native = Vec::new();
    for lane in ["native", "native-replay"] {
        let work = root.join(lane);
        fs::create_dir(&work).unwrap();
        let args = flags
            .iter()
            .copied()
            .chain(["--slice", plate.unwrap_or("0"), "--outputdir", ".", input])
            .map(str::to_owned);
        let result = stages::run_live(runner.bin(), args, stages::Stage::Slice, &work);
        assert_eq!(
            result.map_or_else(|e| e.captured.unwrap().code, |()| Some(0)),
            Some(exit),
            "{work:?}"
        );
        native.push(outputs(&work, exit));
    }
    assert_eq!(native[0].1, native[1].1, "Native result replay: {root:?}");
    if let (Some(first), Some(second)) = (&native[0].0, &native[1].0) {
        assert!(
            normalize_one_generator_line(first, GeneratorKind::Orca).unwrap()
                == normalize_one_generator_line(second, GeneratorKind::Orca).unwrap(),
            "Native G-code replay: {root:?}"
        );
    }

    let work = root.join("ares");
    fs::create_dir(&work).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command
        .current_dir(&work)
        .args(["slice", "-o", "plate_1.gcode"]);
    if let Some(path) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") {
        command.arg("--project-preset-catalog").arg(path);
    }
    if let Some(plate) = plate {
        command.args(["--plate", plate]);
    }
    let result = command.args(flags).arg(input).output().unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "Ares: {work:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = outputs(&work, exit);
    assert_eq!(native[0].1, actual.1, "result.json: {root:?}");
    if let (Some(expected), Some(actual)) = (&native[0].0, &actual.0) {
        compare_ordered_bytes_generator_only(expected, actual)
            .unwrap_or_else(|error| panic!("{root:?}: {error}"));
        if name == "arrange_after_volume_transforms" {
            compare_core_stream(actual, &root);
        }
    }
}

fn compare_core_stream(cli: &[u8], root: &Path) {
    use ares_core::{
        CliProjectSliceRequest, CliProjectTransforms, CliUpwardMachine, CliVolumeTransform,
        ConfigOverrides, GenerationMetadata, SlicedPlate, SlicerContext, UpwardMachineInput,
    };

    let transforms = CliProjectTransforms {
        volume: vec![
            CliVolumeTransform::Scale(1.2),
            CliVolumeTransform::Rotate(45.0),
        ],
        arrange: Some(1),
        ..CliProjectTransforms::NONE
    };
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_cli_project_plate_validated(
            PROJECT,
            CliProjectSliceRequest {
                plate: None,
                overrides: &ConfigOverrides::default(),
                record_moves: false,
                transforms: &transforms,
                upward: CliUpwardMachine {
                    source: None,
                    input: UpwardMachineInput::Embedded,
                    report: &mut Vec::new(),
                },
                sliced_plate: &mut SlicedPlate::default(),
            },
            GenerationMetadata::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("core.gcode"), &output.gcode).unwrap();
    assert!(output.gcode == cli, "raw core API != CLI: {root:?}");
}

fn outputs(work: &Path, exit: i32) -> (Option<Vec<u8>>, Vec<u8>) {
    let stream = fs::read(work.join("plate_1.gcode"));
    if exit != 0 {
        assert_eq!(
            stream.as_ref().unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
    }
    (
        (exit == 0).then(|| stream.unwrap()),
        result_comparison_bytes(&fs::read(work.join("result.json")).unwrap()),
    )
}
