use std::{fs, path::Path};

use serde_json::json;

use super::super::{OrcaRunner, application, join_paths, repo_root, stages};
use super::write_json;

const FIXTURES: &str = "tests/parity/cli-explicit-machine-limits";

#[test]
fn loaded_legacy_wall_order_exports_infill_first_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = repo_root().join(FIXTURES);
    let cube = fixtures.join("cube10.stl");
    let filament = fixtures.join("filament.json");
    let machine = fixtures.join("machine.json");
    let process = runner.work.join("process.json");
    let settings = join_paths([&machine, &process].into_iter());
    let args = [
        "--load-settings",
        &settings,
        "--load-filaments",
        filament.to_str().unwrap(),
    ];
    for (name, order, explicit, expected) in [
        (
            "walls-first",
            "inner wall/outer wall/infill",
            Some("0"),
            "0",
        ),
        ("infill-first", "infill/inner wall/outer wall", None, "1"),
        (
            "derived-precedence",
            "infill/outer wall/inner wall",
            Some("0"),
            "1",
        ),
    ] {
        let mut fields = json!({
            "type": "process",
            "from": "user",
            "name": name,
            "version": "2.5.0.0",
            "wall_infill_order": order,
        });
        if let Some(value) = explicit {
            fields["is_infill_first"] = json!(value);
        }
        write_json(&process, &fields);
        let native = native_infill_first(&runner, name, args, &cube);
        let ares_path = runner.work.join(format!("{name}-ares.3mf"));
        let output = application::cli_command()
            .args(args)
            .args(["--export-3mf", ares_path.to_str().unwrap()])
            .arg(&cube)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual = application::settings(&fs::read(&ares_path).unwrap()).unwrap();
        assert_eq!(native, expected, "{name}");
        assert_eq!(actual["is_infill_first"], native, "{name}");
    }
}

#[test]
fn slice_options_legacy_wall_order_overrides_explicit_infill_first() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = repo_root().join(FIXTURES);
    let machine = fixtures.join("machine.json");
    let filament = fixtures.join("filament.json");
    let process = runner.work.join("process.json");
    write_json(
        &process,
        &json!({
            "type": "process",
            "from": "user",
            "name": "Infill then walls",
            "version": "2.5.0.0",
            "is_infill_first": "0",
            "wall_infill_order": "infill/inner wall/outer wall",
        }),
    );
    let settings = join_paths([&machine, &process].into_iter());
    let native = native_infill_first(
        &runner,
        "selected-order",
        [
            "--load-settings",
            &settings,
            "--load-filaments",
            filament.to_str().unwrap(),
        ],
        &fixtures.join("cube10.stl"),
    );
    assert_eq!(native, "1");
    let options: ares_core::SliceOptions =
        serde_json::from_slice(&fs::read(process).unwrap()).unwrap();
    assert_eq!(options.values()["is_infill_first"], json!(native == "1"));
    assert!(options.is_infill_first().unwrap());
}

fn native_infill_first(
    runner: &OrcaRunner,
    name: &str,
    args: [&str; 4],
    cube: &Path,
) -> serde_json::Value {
    let native_path = runner.work.join(format!("{name}-native.3mf"));
    stages::run_live(
        runner.bin(),
        args.into_iter()
            .chain([
                "--arrange",
                "1",
                "--export-3mf",
                native_path.to_str().unwrap(),
                cube.to_str().unwrap(),
            ])
            .map(str::to_owned),
        stages::Stage::Export,
        &runner.work,
    )
    .unwrap();
    application::settings(&fs::read(native_path).unwrap())
        .unwrap()
        .remove("is_infill_first")
        .unwrap()
}
