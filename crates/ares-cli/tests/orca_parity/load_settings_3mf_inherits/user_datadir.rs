//! Native CLI `OrcaSlicer.cpp:2088–2123`, `PresetBundle.cpp:1411–1425`.
use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use crate::{load_settings_3mf::load_settings_3mf_fixture, runner::stages};

#[test]
fn user_preset_datadir_and_cwd_source_admission() {
    run_case("cwd-only");
    run_case("explicit");
}

#[test]
fn user_preset_selected_user() {
    run_case("selected");
}
#[test]
fn user_preset_wrong_user() {
    run_case("wrong-user");
}
#[test]
fn user_preset_missing_user() {
    run_case("missing-user");
}
#[test]
fn user_preset_invalid_app_config() {
    run_case("invalid-app");
}
#[test]
fn user_preset_external_copy() {
    run_case("copy");
}
#[test]
fn user_preset_missing_parent() {
    run_case("missing-parent");
}
#[test]
fn user_preset_invalid_version() {
    run_case("invalid-version");
}
#[test]
fn user_preset_malformed_json() {
    run_case("malformed");
}

#[test]
fn user_preset_process_and_filament_inherit() {
    run_case("all-kinds");
}

#[test]
fn user_preset_new_datadir_standalone_is_lazy() {
    run_case("new-datadir");
}

#[test]
fn user_preset_sparse_filament_omits_id() {
    run_filament_id_case("omitted-id", None, r#"; filament_ids = """#);
}

#[test]
fn user_preset_sparse_filament_keeps_empty_id() {
    run_filament_id_case("empty-id", Some(""), r#"; filament_ids = """#);
}

#[test]
fn user_preset_sparse_filament_keeps_authored_id() {
    run_filament_id_case("authored-id", Some("USER99"), "; filament_ids = USER99");
}

fn run_case(case: &str) {
    run(case, None);
}

fn run_filament_id_case(case: &str, authored: Option<&str>, expected: &str) {
    run(case, Some((authored, expected)));
}

fn run(case: &str, filament_id_case: Option<(Option<&str>, &str)>) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("user-datadir-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let mut process = load_settings_3mf_fixture("process.json");
    let mut filament = load_settings_3mf_fixture("compatible-filament-0.json");
    let explicit = case != "cwd-only";
    let success = filament_id_case.is_some()
        || matches!(
            case,
            "explicit" | "selected" | "invalid-app" | "malformed" | "new-datadir" | "all-kinds"
        );
    let cwd = root.join(case);
    fs::create_dir_all(cwd.join("system")).unwrap();
    let user = cwd.join(if case == "selected" {
        "user/chosen"
    } else {
        "user/default"
    });
    if matches!(case, "selected" | "wrong-user" | "missing-user") {
        fs::write(
            cwd.join("OrcaSlicer.conf"),
            json!({"app":{"preset_folder":"chosen"}}).to_string(),
        )
        .unwrap();
    } else if case == "invalid-app" {
        fs::write(cwd.join("OrcaSlicer.conf"), "{ invalid json").unwrap();
    }
    for directory in [
        "machine/base",
        "process",
        "print",
        "filament",
        "home",
        "tmp",
    ] {
        fs::create_dir_all(user.join(directory)).unwrap();
    }
    let mut parent: Value =
        serde_json::from_slice(&fs::read(load_settings_3mf_fixture("machine.json")).unwrap())
            .unwrap();
    parent["version"] = "2.5.0".into();
    parent["from"] = "user".into();
    let parent_name = parent["name"].as_str().unwrap();
    fs::write(
        user.join(format!("machine/base/{parent_name}.json")),
        parent.to_string(),
    )
    .unwrap();
    let mut source = user.join("machine/user-machine.json");
    let mut child: Value =
        serde_json::from_slice(&fs::read(load_settings_3mf_fixture("user-machine.json")).unwrap())
            .unwrap();
    child["version"] = if case == "invalid-version" {
        "not-a-version"
    } else {
        "2.5.0"
    }
    .into();
    if case == "missing-parent" {
        child["inherits"] = "Missing parent".into();
    }
    if case == "new-datadir" {
        child = parent.clone();
        child["from"] = "system".into();
        child.as_object_mut().unwrap().remove("inherits");
    }
    fs::write(&source, child.to_string()).unwrap();
    if case == "copy" {
        let external = cwd.join("external.json");
        fs::copy(&source, &external).unwrap();
        source = external;
    }
    if case == "malformed" {
        fs::write(user.join("machine/broken.json"), "{ invalid json").unwrap();
    }
    for (directory, fixture) in [("process", &process), ("filament", &filament)] {
        let mut document: Value = serde_json::from_slice(&fs::read(fixture).unwrap()).unwrap();
        document["version"] = "2.5.0".into();
        document["from"] = "user".into();
        fs::create_dir_all(user.join(directory).join("base")).unwrap();
        fs::write(
            user.join(directory).join("base/root.json"),
            document.to_string(),
        )
        .unwrap();
    }
    for (kind, file) in [("process", &mut process), ("filament", &mut filament)] {
        if case != "all-kinds" && !(kind == "filament" && filament_id_case.is_some()) {
            continue;
        }
        let template: Value = serde_json::from_slice(&fs::read(&*file).unwrap()).unwrap();
        let mut child = json!({"type":kind,"name":template["name"],"from":"user","version":"2.5.0","inherits":"root"});
        if kind == "filament" {
            match filament_id_case {
                None => child["filament_id"] = template["filament_id"].clone(),
                Some((Some(id), _)) => child["filament_id"] = id.into(),
                Some((None, _)) => {}
            }
        }
        *file = user.join(kind).join("child.json");
        fs::write(&*file, child.to_string()).unwrap();
    }
    let settings = format!("{};{}", source.display(), process.display());
    let output = cwd.join("native-output");
    fs::create_dir(&output).unwrap();
    let mut argv = vec![
        "--slice".to_owned(),
        "0".to_owned(),
        "--load-settings".to_owned(),
        settings.clone(),
        "--load-filaments".to_owned(),
        filament.display().to_string(),
        "--arrange".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.display().to_string(),
        project.display().to_string(),
    ];
    if explicit {
        let datadir = if case == "new-datadir" {
            cwd.join("new-native")
        } else {
            cwd.clone()
        };
        argv.extend(["--datadir".to_owned(), datadir.display().to_string()]);
    }
    let home = user.join("home");
    let tmp = user.join("tmp");
    let envs = [
        ("HOME", home.to_str().unwrap()),
        ("XDG_CONFIG_HOME", home.to_str().unwrap()),
        ("XDG_CACHE_HOME", home.to_str().unwrap()),
        ("TMPDIR", tmp.to_str().unwrap()),
    ];
    let native = stages::run_exact_with_env(runner.bin(), argv, &envs, stages::Stage::Slice, &cwd);
    let result: Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    fs::write(
        cwd.join("native-observation.json"),
        json!({
            "explicit_datadir":explicit, "success":native.is_ok(), "result":result,
        })
        .to_string(),
    )
    .unwrap();
    if success {
        assert!(native.is_ok(), "{}: {}", cwd.display(), native.unwrap_err());
        assert_eq!(result["return_code"], 0, "{}", cwd.display());
        let gcode = output.join("plate_1.gcode");
        assert!(gcode.is_file());
        if let Some((_, expected)) = filament_id_case {
            assert!(
                fs::read_to_string(gcode)
                    .unwrap()
                    .lines()
                    .any(|line| line == expected),
                "Native did not emit {expected:?}: {}",
                cwd.display()
            );
        }
    } else {
        let failure = native.unwrap_err();
        assert_eq!(failure.kind, stages::FailureKind::Process);
        let captured = failure.captured.unwrap();
        assert_eq!(captured.code, Some(251));
        assert_eq!(captured.stdout, b"run found error, return -5, exit...\n");
        assert!(String::from_utf8_lossy(&captured.stderr).contains("can not resolve preset"));
        assert_eq!(result["return_code"], -5);
        assert!(!output.join("plate_1.gcode").exists());
    }
    ares(
        &cwd,
        [&source, &process, &filament, &project],
        &envs,
        (explicit, success),
    );
}

fn ares(
    cwd: &Path,
    [source, process, filament, project]: [&Path; 4],
    envs: &[(&str, &str)],
    (explicit, success): (bool, bool),
) {
    let output = cwd.join("ares-output/ares.gcode");
    fs::create_dir(output.parent().unwrap()).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_ares"));
    command.current_dir(cwd).envs(envs.iter().copied()).args([
        "slice",
        "--load-settings",
        &format!("{};{}", source.display(), process.display()),
        "--load-filaments",
        filament.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        project.to_str().unwrap(),
    ]);
    if explicit {
        let datadir = if cwd.ends_with("new-datadir") {
            cwd.join("new-ares")
        } else {
            cwd.to_owned()
        };
        command.arg("--datadir").arg(datadir);
    }
    let result = command.output().unwrap();
    fs::write(cwd.join("ares.stdout"), &result.stdout).unwrap();
    fs::write(cwd.join("ares.stderr"), &result.stderr).unwrap();
    fs::write(
        cwd.join("ares-status.json"),
        json!({"code":result.status.code()}).to_string(),
    )
    .unwrap();
    if success {
        if cwd.ends_with("new-datadir") {
            assert!(cwd.join("new-native").is_dir());
            assert!(
                cwd.join("new-ares").is_dir(),
                "Ares must create --datadir during setup"
            );
        }
        assert!(
            result.status.success(),
            "{}: {}",
            cwd.display(),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(output.is_file());
        let ares_result: Value =
            serde_json::from_slice(&fs::read(output.with_file_name("result.json")).unwrap())
                .unwrap();
        assert_eq!(ares_result["return_code"], 0, "{}", cwd.display());
        let native = fs::read(cwd.join("native-output/plate_1.gcode")).unwrap();
        let ares = fs::read(&output).unwrap();
        let native = crate::load_settings_cli::banner_normalized(&native);
        let ares = crate::load_settings_cli::banner_normalized(&ares);
        fs::write(cwd.join("native-normalized.gcode"), &native).unwrap();
        fs::write(cwd.join("ares-normalized.gcode"), &ares).unwrap();
        assert!(
            native == ares,
            "user producer whole-file divergence: {}",
            cwd.display()
        );
    } else {
        assert_eq!(result.status.code(), Some(251));
        assert_eq!(result.stdout, b"run found error, return -5, exit...\n");
        assert!(!output.exists());
    }
}
