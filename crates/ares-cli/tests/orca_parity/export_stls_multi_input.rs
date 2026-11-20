use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use super::export_stl::{synthetic_3mf, synthetic_stl, write_native_presets};
use crate::OrcaRunner;

#[test]
fn cli_export_stl_merges_two_stls_in_positional_order() {
    compare_exports(false, false, false);
}

#[test]
fn cli_export_stl_merges_two_stls_with_explicit_presets() {
    compare_exports(false, false, true);
}

#[test]
fn cli_export_stls_merges_two_stls_in_the_explicit_directory() {
    compare_exports(true, false, false);
}

#[test]
fn cli_export_stls_merges_two_stls_with_explicit_presets() {
    compare_exports(true, false, true);
}

#[test]
fn cli_export_stl_preserves_plain_3mf_build_order_after_stl() {
    compare_exports(false, true, false);
}

#[test]
fn cli_export_stl_merges_stl_and_plain_3mf_with_explicit_presets() {
    compare_exports(false, true, true);
}

#[test]
fn cli_export_stls_preserves_plain_3mf_build_order_after_stl() {
    compare_exports(true, true, false);
}

#[test]
fn cli_export_stls_merges_stl_and_plain_3mf_with_explicit_presets() {
    compare_exports(true, true, true);
}

#[test]
#[cfg(unix)]
fn cli_export_stls_rejects_missing_and_wrong_type_presets_before_writing_meshes() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    write_inputs(runner.work_dir());
    for (case, flags, code, stderr) in [
        (
            "settings",
            ["--load-settings", "missing.json"],
            253,
            b"operator(): can not find setting file: missing.json\n".as_slice(),
        ),
        (
            "filaments",
            ["--load-filaments", "missing.json"],
            253,
            b"operator(): can not find setting file: missing.json\n".as_slice(),
        ),
        (
            "wrong-type",
            ["--load-filaments", "../machine.json"],
            251,
            b"".as_slice(),
        ),
    ] {
        let args = [
            "--export-stls",
            "custom/deep",
            flags[0],
            flags[1],
            "../Zed.stl",
            "../Scene.3mf",
        ];
        compare(&runner, case, &args, (Some(code), stderr), &[]);
    }
}

fn compare_exports(plural: bool, mixed: bool, presets: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    write_inputs(runner.work_dir());
    let (mut args, destination) = if plural {
        (vec!["--export-stls", "custom/deep"], "custom/deep")
    } else {
        (vec!["--export-stl"], "report/stl")
    };
    if presets {
        args.extend([
            "--load-settings",
            "../machine.json;../process.json",
            "--load-filaments",
            "../filament.json",
        ]);
    }
    args.push("../Zed.stl");
    let names = if mixed {
        args.push("../Scene.3mf");
        ["obj_1_Zed.stl", "obj_2_Beta.stl", "obj_3_Alpha.stl"].as_slice()
    } else {
        args.push("../Aye.stl");
        ["obj_1_Zed.stl", "obj_2_Aye.stl"].as_slice()
    };
    let expected = names
        .iter()
        .map(|name| Path::new(destination).join(name))
        .collect::<Vec<_>>();
    compare(&runner, "export", &args, (Some(0), b""), &expected);
}

fn write_inputs(dir: &Path) {
    let mut stl = synthetic_stl();
    fs::write(dir.join("Zed.stl"), &stl).unwrap();
    // Give the second mesh different geometry despite sharing the first mesh's topology.
    for facet in stl[84..].chunks_exact_mut(50) {
        for x in [12, 24, 36] {
            let value = f32::from_le_bytes(facet[x..x + 4].try_into().unwrap());
            facet[x..x + 4].copy_from_slice(&(value * 2.0 + 24.0).to_le_bytes());
        }
    }
    fs::write(dir.join("Aye.stl"), stl).unwrap();
    fs::write(dir.join("Scene.3mf"), synthetic_3mf(false, false)).unwrap();
    write_native_presets(dir);
}

fn compare(
    runner: &OrcaRunner,
    case: &str,
    args: &[&str],
    (code, stderr): (Option<i32>, &[u8]),
    stls: &[PathBuf],
) {
    let root = runner.work_dir();
    let run = |side: &str, mut command: Command| {
        let dir = root.join(format!("{case}-{side}"));
        fs::create_dir_all(dir.join("report")).unwrap();
        let Output {
            status,
            stdout,
            stderr,
        } = command
            .current_dir(&dir)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .args(["--arrange", "0", "--orient", "0", "--outputdir", "report"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.with_extension("stdout"), stdout).unwrap();
        fs::write(dir.with_extension("stderr"), &stderr).unwrap();
        fs::write(dir.with_extension("exit"), format!("{:?}\n", status.code())).unwrap();
        assert_eq!(
            status.code(),
            code,
            "{side} {args:?}: {}",
            String::from_utf8_lossy(&stderr)
        );
        let files = inventory(&dir);
        (dir, files, stderr)
    };
    // Native prunes the shared model-backup tree at exit. Give each invocation
    // its own temp root and route diagnostic logging outside the artifact tree.
    let temp = root.join(format!("{case}-native-tmp"));
    fs::create_dir(&temp).unwrap();
    let mut orca = Command::new(runner.bin());
    orca.env("TMPDIR", &temp)
        .env("TMP", &temp)
        .env("TEMP", &temp)
        .arg("--logfile")
        .arg(root.join(format!("{case}-native.log")));
    let (native, expected_files, expected_stderr) = run("native", orca);
    assert_eq!(expected_stderr, stderr, "Native stderr");
    let mut expected = stls.to_vec();
    if cfg!(target_os = "linux") {
        expected.push("report/result.json".into());
    }
    expected.sort();
    assert_eq!(expected_files, expected, "Native inventory");

    let mut cli = Command::new(env!("CARGO_BIN_EXE_ares"));
    cli.arg("slice");
    let (ares, actual_files, actual_stderr) = run("ares", cli);
    assert_eq!(actual_stderr, expected_stderr, "stderr");
    assert_eq!(actual_files, expected_files, "complete file inventory");
    for path in expected_files {
        let expected = fs::read(native.join(&path)).unwrap();
        let actual = fs::read(ares.join(&path)).unwrap();
        if path.ends_with("result.json") {
            assert_eq!(report(&actual), report(&expected), "{path:?}");
        } else {
            assert_eq!(actual.len(), expected.len(), "{path:?}");
            assert_eq!(
                actual.iter().zip(expected).position(|(a, b)| *a != b),
                None,
                "first differing byte in {path:?}"
            );
        }
    }
}

fn inventory(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_owned()];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                dirs.push(entry.path());
            } else {
                files.push(entry.path().strip_prefix(root).unwrap().to_owned());
            }
        }
    }
    files.sort();
    files
}

fn report(bytes: &[u8]) -> serde_json::Value {
    let mut report: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    for time in ["prepare_time", "export_time"] {
        assert!(report[time].is_u64(), "expected an unsigned {time}");
        report[time] = 0.into();
    }
    report
}
