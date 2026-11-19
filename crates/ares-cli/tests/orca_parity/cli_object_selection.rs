#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use super::cli_project_transforms::outputs;
use crate::{
    golden::{GeneratorKind, compare_ordered_bytes_generator_only, normalize_one_generator_line},
    runner::{OrcaRunner, stages},
};

// Two separated cubes: 3MF resource IDs 2 and 3, loaded instance IDs 14 and 73.
const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_object_selection.3mf");

enum Witness {
    RetainsLeftOnly,
    Unchanged,
    AllSkipped,
    Rejected(&'static str),
}

use Witness::{AllSkipped, Rejected, RetainsLeftOnly, Unchanged};

const ASSIGNMENT_REJECTED: &str = "can not load 3mf when set loaded_filament_ids or clone_objects";

macro_rules! case {
    ($name:ident, $outcome:expr, [$($arg:literal),*]) => {
        #[test]
        fn $name() {
            verify(stringify!($name), $outcome, &[$($arg),*]);
        }
    };
}

case!(
    skips_one_saved_id,
    RetainsLeftOnly,
    ["--skip-objects", "73"]
);
case!(
    resource_ids_do_not_skip_instances,
    Unchanged,
    ["--skip-objects", "2,3"]
);
case!(
    repeated_skip_lists_can_empty_a_plate,
    AllSkipped,
    ["--skip-objects", "14,999", "--skip-objects=73"]
);
case!(
    clone_count_is_checked_first,
    Rejected("clone_objects size 2 should be the same with input files size 1"),
    ["--clone-objects", "1,2", "--load-filament-ids", "1,2"]
);
case!(
    clone_requires_a_filament_list,
    Rejected("clone_objects should be used with load_filaments together"),
    ["--clone-objects", "1"]
);
case!(
    saved_project_rejects_clone_after_prerequisites,
    Rejected(ASSIGNMENT_REJECTED),
    ["--clone-objects", "1", "--load-filaments", "not-read.json"]
);
case!(
    filament_id_count_is_checked_first,
    Rejected("loaded_filament_ids size 2 should be the same with input files size 1"),
    ["--load-filament-ids", "1,2"]
);
case!(
    filament_ids_require_a_filament_list,
    Rejected("loaded_filament_ids should be used with load_filaments together"),
    ["--load-filament-ids", "1"]
);
case!(
    saved_project_rejects_filament_ids_after_prerequisites,
    Rejected(ASSIGNMENT_REJECTED),
    [
        "--load-filament-ids",
        "1",
        "--load-filaments",
        "not-read.json"
    ]
);

fn verify(name: &str, witness: Witness, flags: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("cli-object-selection-{name}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("project.3mf");
    fs::write(&input, PROJECT).unwrap();
    let input = input.to_str().unwrap();
    let exit = match witness {
        RetainsLeftOnly | Unchanged => 0,
        AllSkipped => 196,
        Rejected(_) => 254,
    };
    let primary = root.join("native");
    let replay = root.join("native-replay");
    let (native, report) = run_native(&runner, &primary, input, flags, exit);
    let (again, again_report) = run_native(&runner, &replay, input, flags, exit);
    assert_eq!(report, again_report, "Native report replay: {root:?}");
    assert!(
        normalized(&native) == normalized(&again),
        "Native G-code replay: {root:?}"
    );
    match witness {
        RetainsLeftOnly | Unchanged => {
            let (control, _) = run_native(&runner, &root.join("native-control"), input, &[], 0);
            let (native, control) = (normalized(&native).unwrap(), normalized(&control).unwrap());
            let gcode = std::str::from_utf8(&native).unwrap();
            let baseline = std::str::from_utf8(&control).unwrap();
            for object in ["left.stl", "right.stl"] {
                assert!(
                    prints_object(baseline, object),
                    "Native control lacks {object}: {root:?}"
                );
            }
            assert!(
                prints_object(gcode, "left.stl"),
                "Native lost left object: {root:?}"
            );
            if matches!(witness, RetainsLeftOnly) {
                assert!(
                    !prints_object(gcode, "right.stl"),
                    "Native kept skipped object: {root:?}"
                );
            } else {
                assert!(
                    native == control,
                    "Unknown IDs changed Native output: {root:?}"
                );
            }
        }
        Rejected(detail) => {
            for work in [&primary, &replay] {
                check_diagnostic(
                    &fs::read_to_string(work.join("00000.log")).unwrap(),
                    detail,
                    work,
                );
            }
        }
        AllSkipped => {}
    }

    let work = root.join("ares");
    fs::create_dir(&work).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command
        .current_dir(&work)
        .args(["slice", "--plate", "1", "-o", "plate_1.gcode"]);
    // The assignment rejections occur before preset loading. Do not add an
    // Ares-only catalog argument that conflicts with --load-filaments.
    if !matches!(witness, Rejected(_))
        && let Some(path) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG")
    {
        command.arg("--project-preset-catalog").arg(path);
    }
    let result = command.args(flags).arg(input).output().unwrap();
    fs::write(work.join("stdout"), &result.stdout).unwrap();
    fs::write(work.join("stderr"), &result.stderr).unwrap();
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.code(), Some(exit), "Ares: {work:?}: {stderr}");
    if let Rejected(detail) = witness {
        check_diagnostic(&stderr, detail, &work);
    }
    let (actual, actual_report) = outputs(&work, exit);
    assert_eq!(report, actual_report, "result.json: {root:?}");
    if let (Some(native), Some(actual)) = (native, actual) {
        compare_ordered_bytes_generator_only(&native, &actual)
            .unwrap_or_else(|error| panic!("{root:?}: {error}"));
    }
}

fn prints_object(stream: &str, name: &str) -> bool {
    stream.contains(&format!("; printing object {name} "))
}

fn check_diagnostic(output: &str, detail: &str, work: &Path) {
    assert!(
        output.contains(detail),
        "{work:?}: expected diagnostic {detail:?}"
    );
}

fn normalized(stream: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    stream
        .as_ref()
        .map(|bytes| normalize_one_generator_line(bytes, GeneratorKind::Orca).unwrap())
}

fn run_native(
    runner: &OrcaRunner,
    work: &Path,
    input: &str,
    flags: &[&str],
    exit: i32,
) -> (Option<Vec<u8>>, Vec<u8>) {
    fs::create_dir(work).unwrap();
    let args = flags
        .iter()
        .copied()
        .chain(["--slice", "1", "--outputdir", ".", input])
        .map(str::to_owned);
    let result = stages::run_live(runner.bin(), args, stages::Stage::Slice, work);
    assert_eq!(
        result.map_or_else(|error| error.captured.unwrap().code, |()| Some(0)),
        Some(exit),
        "Native: {work:?}"
    );
    outputs(work, exit)
}
