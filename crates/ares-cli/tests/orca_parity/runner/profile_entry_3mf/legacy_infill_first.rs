use std::fs;

use serde_json::json;

use super::super::{OrcaRunner, application, join_paths, repo_root, stages};
use super::write_json;

#[test]
fn loaded_legacy_wall_order_exports_infill_first_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = repo_root().join("tests/parity/cli-explicit-machine-limits");
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
        let ares_path = runner.work.join(format!("{name}-ares.3mf"));
        let output = application::cli_command()
            .arg("slice")
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
        let native = application::settings(&fs::read(&native_path).unwrap()).unwrap();
        let actual = application::settings(&fs::read(&ares_path).unwrap()).unwrap();
        assert_eq!(native["is_infill_first"], expected, "{name}");
        assert_eq!(
            actual["is_infill_first"], native["is_infill_first"],
            "{name}"
        );
    }
}
