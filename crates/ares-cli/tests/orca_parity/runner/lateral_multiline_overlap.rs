//! Public RED for `FillRectilinear.cpp:2960-2994,3031-3032`.
//! Native G-code is produced at runtime from frozen input-only 3MF fixtures.

use super::OrcaRunner;
use crate::runner;
use std::{fs, process::Command};

struct Case {
    name: &'static str,
    input: &'static [u8],
}

macro_rules! case_test {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            compare(Case {
                name: stringify!($name),
                input: include_bytes!(concat!(
                    "../../../../../tests/parity/lateral-multiline/",
                    $file,
                    ".3mf"
                )),
            });
        }
    };
}

case_test!(lateral_lattice_m2, "ll-m2-9pzw01");
case_test!(lateral_lattice_m3, "ll-m3-PdvvMh");
case_test!(lateral_lattice_n2, "ll-n2-1BOiIb");
case_test!(lateral_lattice_n3, "ll-n3-MoMLQy");
case_test!(lateral_honeycomb_m2, "lh-m2-uPaaYW");
case_test!(lateral_honeycomb_m3, "lh-m3-0txUVn");
case_test!(lateral_lattice_m1, "ll-m1-Pc0Ov5");
case_test!(lateral_honeycomb_m1, "lh-m1-q66gGy");
case_test!(cubic_m2, "c-m2-3o18mj");
case_test!(quartercubic_m2, "q-m2-8zAhcp");

fn compare(case: Case) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("lateral-multiline-red-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    fs::write(&input, case.input).unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    runner::stages::run(
        runner.bin(),
        [
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            native_dir.to_str().unwrap().into(),
            input.to_str().unwrap().into(),
        ],
        runner::stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let native_result: serde_json::Value =
        serde_json::from_slice(&fs::read(native_dir.join("result.json")).unwrap()).unwrap();
    assert_eq!(native_result["return_code"], 0, "{}", case.name);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(ares_core::slice_project_plate_validated(
        case.input,
        ares_core::GenerationMetadata::deterministic(2026, 10, 15, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let api_comparison = api
        .as_ref()
        .map_err(|error| format!("{error:?}"))
        .and_then(|bytes| crate::golden::compare_ordered_bytes_generator_only(&native, bytes));

    let cli_output = root.join("ares.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_output)
        .arg(&input)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{}: {:?}", case.name, cli);
    let cli_bytes = fs::read(&cli_output).unwrap();
    let cli_comparison = crate::golden::compare_ordered_bytes_generator_only(&native, &cli_bytes);

    assert_eq!(api_comparison, Ok(()), "{} API", case.name);
    assert_eq!(cli_comparison, Ok(()), "{} CLI", case.name);
}
