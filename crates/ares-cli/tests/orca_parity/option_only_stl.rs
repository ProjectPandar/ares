use std::{fs, path::Path, process::Command};

use serde_json::json;

use crate::{
    OrcaRunner, artifacts, golden,
    runner::{
        native_api_cli::sha,
        stages::{self, Stage},
    },
};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/cube10.stl");
const RESET: &str = "G92 E0";
const NAME: &str = "input.stl";

#[test]
fn grounded_default_monotonicline() {
    check("grounded", 0.0, None);
}

#[test]
fn explicit_layer_height_overrides_default() {
    check("explicit-height", 0.0, Some(0.25));
}

#[test]
fn above_bed_drops_to_ground() {
    check("above-bed", 1.0, None);
}

#[test]
fn below_bed_rises_to_ground() {
    check("below-bed", -1.0, None);
}

fn check(label: &str, min_z: f32, layer_height: Option<f64>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("option-only-stl-{label}-"))
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let stl = translated_cube(min_z);
    let input = root.join(NAME);
    let options_path = root.join("options.json");
    let mut options = json!({"before_layer_change_gcode": RESET});
    if let Some(height) = layer_height {
        options["layer_height"] = json!(height);
    }
    fs::write(&input, &stl).unwrap();
    fs::write(&options_path, serde_json::to_vec(&options).unwrap()).unwrap();

    let native = native_slice(&runner, &root, &input, "native", layer_height);
    let replay = native_slice(&runner, &root, &input, "replay", layer_height);
    let normalize_orca =
        |bytes| golden::normalize_one_generator_line(bytes, golden::GeneratorKind::Orca).unwrap();
    let first = normalize_orca(&native);
    let second = normalize_orca(&replay);
    let replay_difference = golden::first_difference(&first, &second).map(|v| v.to_string());
    assert!(
        replay_difference.is_none(),
        "{root:?}: {replay_difference:?}"
    );
    check_native_config(&native, layer_height);

    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_stl_validated(
            &stl,
            serde_json::from_value::<ares_core::SliceOptions>(options.clone()).unwrap(),
        ));
    let cli_path = root.join("cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--options"])
        .arg(&options_path)
        .arg("-o")
        .arg(&cli_path)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI failed at {root:?}");
    let api = api.unwrap_or_else(|error| panic!("API failed at {root:?}: {error}"));
    fs::write(root.join("api.gcode"), &api).unwrap();
    let cli = fs::read(cli_path).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&native, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&native, &cli);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label, "stl_sha256": sha(&stl), "options": options,
            "native_replay_sha256": sha(&first), "native_replay_difference": replay_difference,
            "api_cli_equal": api == cli, "api_comparison": api_comparison,
            "cli_comparison": cli_comparison,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(api == cli, "API/CLI byte mismatch at {root:?}");
    assert_eq!(api_comparison, Ok(()), "Native/API at {root:?}");
    assert_eq!(cli_comparison, Ok(()), "Native/CLI at {root:?}");
}

fn translated_cube(min_z: f32) -> Vec<u8> {
    let mut bytes = CUBE.to_vec();
    let triangles = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 84 + triangles * 50);
    for triangle in bytes[84..].as_chunks_mut::<50>().0 {
        for offset in [20, 32, 44] {
            let z = &mut triangle[offset..offset + 4];
            let old = f32::from_le_bytes(z.try_into().unwrap());
            assert!(matches!(old, -5.0 | 5.0));
            z.copy_from_slice(&(old + 5.0 + min_z).to_le_bytes());
        }
    }
    bytes
}

fn check_native_config(native: &[u8], layer_height: Option<f64>) {
    let text = std::str::from_utf8(native).unwrap();
    assert!(
        text.lines()
            .any(|line| line == "; top_surface_pattern = monotonicline")
    );
    let expected = match layer_height {
        Some(_) => "; layer_height = 0.25",
        None => "; layer_height = 0.2",
    };
    assert!(text.lines().any(|line| line == expected), "{expected}");
    let z = text.lines().find_map(|line| line.strip_prefix(";Z:"));
    assert_eq!(z, Some("0.2"), "first layer is on the bed");
}

fn native_slice(
    runner: &OrcaRunner,
    root: &Path,
    input: &Path,
    stage: &str,
    layer_height: Option<f64>,
) -> Vec<u8> {
    let output = root.join(stage);
    fs::create_dir(&output).unwrap();
    let mut args: Vec<String> = ["--slice", "0", "--before-layer-change-gcode", RESET]
        .into_iter()
        .map(str::to_owned)
        .collect();
    if let Some(height) = layer_height {
        args.extend(["--layer-height".to_owned(), height.to_string()]);
    }
    args.extend([
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        input.to_str().unwrap().to_owned(),
    ]);
    stages::run_live(runner.bin(), args, Stage::Slice, &output).unwrap();
    fs::read(output.join("plate_1.gcode")).unwrap()
}
