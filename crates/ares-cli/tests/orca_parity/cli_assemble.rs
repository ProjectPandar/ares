#![cfg(target_os = "linux")]

#[path = "cli_assemble/fixture.rs"]
mod fixture;

use std::{fs, path::Path, process::Output};

use crate::{load_settings_cli::preset_files, runner::OrcaRunner};
use serde_json::{Value, json};

fn native(runner: &OrcaRunner, root: &Path, args: &[&str]) -> Output {
    fs::create_dir(root.join("native")).unwrap();
    let result = std::process::Command::new(runner.bin())
        .current_dir(root)
        .args(["--outputdir", "native", "--slice", "0"])
        .args(args)
        .output()
        .unwrap();
    fs::write(root.join("native.stdout"), &result.stdout).unwrap();
    fs::write(root.join("native.stderr"), &result.stderr).unwrap();
    result
}

fn ares(root: &Path, output: &str, args: &[&str]) -> Output {
    let result = crate::runner::application::cli_command()
        .current_dir(root)
        .args(["slice", "-o", output])
        .args(args)
        .output()
        .unwrap();
    fs::write(root.join(format!("{output}.stdout")), &result.stdout).unwrap();
    fs::write(root.join(format!("{output}.stderr")), &result.stderr).unwrap();
    result
}

fn success(output: Output, root: &Path) {
    assert!(
        output.status.success(),
        "{}: {:?}\n{}\n{}",
        root.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn case(name: &str) -> std::path::PathBuf {
    tempfile::Builder::new()
        .prefix(&format!("assembly-{name}-"))
        .tempdir()
        .unwrap()
        .keep()
}

fn presets(root: &Path, vendor: &str, printer: &str) -> (String, String) {
    let paths = preset_files(root, vendor, printer);
    (paths[..2].join(";"), paths[2].clone())
}

fn compare_geometry(root: &Path, (native_plate, ares_output): (u32, &str)) {
    let read = |path: &str| fs::read_to_string(root.join(path)).unwrap();
    let native = fixture::outer_wall_extents(&read(&format!("native/plate_{native_plate}.gcode")));
    let actual = fixture::outer_wall_extents(&read(ares_output));
    assert!(!native.is_empty(), "no outer walls at {}", root.display());
    assert_eq!(
        actual,
        native,
        "outer wall extents per layer at {}",
        root.display()
    );
}

#[test]
fn cli_assemble_preserves_two_stl_volume_placements() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = case("stl");
    let (settings, filament) = presets(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    fs::write(root.join("post.stl"), fixture::stl(40.0, 45.0)).unwrap();
    fs::write(root.join("side.stl"), fixture::stl(58.0, 50.0)).unwrap();
    let args = [
        "--assemble",
        "--arrange",
        "0",
        "--load-settings",
        &settings,
        "--load-filaments",
        &filament,
        "--load-filament-ids",
        "1,1",
        "post.stl",
        "side.stl",
    ];
    success(native(&runner, &root, &args), &root);
    success(ares(&root, "ares.gcode", &args), &root);
    compare_geometry(&root, (1, "ares.gcode"));
}

#[test]
fn cli_assemble_keeps_component_transforms_not_build_transforms() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = case("project");
    let (settings, filament) = presets(&root, "BBL", "Bambu Lab A1 0.4 nozzle");
    fixture::project(&root.join("parts.3mf"));
    let args = [
        "--assemble",
        "--arrange",
        "0",
        "--load-settings",
        &settings,
        "--load-filaments",
        &filament,
        "parts.3mf",
    ];
    success(native(&runner, &root, &args), &root);
    success(ares(&root, "ares.gcode", &args), &root);
    compare_geometry(&root, (1, "ares.gcode"));
}

#[test]
fn cli_assemble_list_groups_and_translates_independently_on_each_plate() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = case("plates");
    let (settings, filament) = presets(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    fs::write(root.join("post.stl"), fixture::stl(40.0, 45.0)).unwrap();
    fs::write(root.join("side.stl"), fixture::stl(60.0, 55.0)).unwrap();
    let plate = |name, x, y| {
        json!({"plate_name":name,"need_arrange":false,"objects":[
            {"path":"post.stl","count":2,"filaments":[1],"assemble_index":[7],"pos_x":[x, 10.0],"pos_y":[y, 0.0],"pos_z":[0.0, 0.0]},
            {"path":"side.stl","count":1,"filaments":[1],"assemble_index":[7],"pos_x":[x],"pos_y":[y]}
        ]})
    };
    fs::write(
        root.join("list.json"),
        serde_json::to_vec(
            &json!({"plates":[plate("first", 4.0, 5.0),plate("second", 17.0, 19.0)]}),
        )
        .unwrap(),
    )
    .unwrap();
    let args = [
        "--load-assemble-list",
        "list.json",
        "--load-settings",
        &settings,
        "--load-filaments",
        &filament,
    ];
    success(native(&runner, &root, &args), &root);
    for (plate, output) in [(1, "one.gcode"), (2, "two.gcode")] {
        let mut args = args.to_vec();
        let plate_arg = plate.to_string();
        args.extend(["--plate", &plate_arg]);
        success(ares(&root, output, &args), &root);
        compare_geometry(&root, (plate, output));
    }
}

#[test]
fn cli_assemble_rejects_native_invalid_requests() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let empty = &["--assemble"][..];
    let conflict = &["--load-assemble-list", "list.json", "--assemble"][..];
    let malformed = &["--load-assemble-list", "list.json"][..];
    let missing = &["--load-assemble-list", "missing.json"][..];
    for (name, args, code) in [
        ("empty", empty, -2),
        ("conflict", conflict, -2),
        ("malformed", malformed, -5),
        ("missing", missing, -3),
    ] {
        let root = case(name);
        fs::write(root.join("list.json"), b"{\"plates\":[]}").unwrap();
        let oracle = native(&runner, &root, args);
        assert_eq!(oracle.status.code(), Some(256 + code), "{root:?}");
        let actual = ares(&root, "ares.gcode", args);
        assert_eq!(
            actual.status.code(),
            oracle.status.code(),
            "{root:?}: {}",
            String::from_utf8_lossy(&actual.stderr)
        );
        let result =
            |path| serde_json::from_slice::<Value>(&fs::read(root.join(path)).unwrap()).unwrap();
        let (expected, actual) = (result("native/result.json"), result("result.json"));
        for key in ["return_code", "error_string", "plate_index"] {
            assert_eq!(actual[key], expected[key], "{key} at {root:?}");
        }
        assert!(!root.join("ares.gcode").exists());
    }
}
