use std::{fs, path::Path};

use serde_json::Value;

use super::{ARGS, OrcaRunner, golden, run_ares, stages};

#[test]
fn prints_along_belt_copies_at_their_own_layer_heights() {
    compare(include_bytes!(
        "../../../../../tests/parity/belt-placement/along.3mf"
    ));
}

#[test]
fn visits_both_across_belt_copies_on_each_shared_layer() {
    compare(include_bytes!(
        "../../../../../tests/parity/belt-placement/across.3mf"
    ));
}

#[test]
fn interleaves_the_separate_layer_heights_of_staggered_copies() {
    compare(include_bytes!(
        "../../../../../tests/parity/belt-placement/staggered.3mf"
    ));
}

#[test]
fn prints_distinct_belt_meshes_with_their_own_height_and_placement() {
    compare(include_bytes!(
        "../../../../../tests/parity/belt-placement/distinct.3mf"
    ));
}

#[test]
fn follows_shifted_layer_height_after_reversing_model_order() {
    compare(include_bytes!(
        "../../../../../tests/parity/belt-placement/along-reverse.3mf"
    ));
}

#[test]
fn rejects_active_brim_on_a_plate_with_multiple_belt_placements() {
    rejects(
        include_bytes!("../../../../../tests/parity/belt-placement/across-brim.3mf"),
        "multi_placement_brim",
    );
}

#[test]
fn rejects_by_object_sequence_for_multiple_belt_placements() {
    rejects(
        include_bytes!("../../../../../tests/parity/belt-placement/by-object.3mf"),
        "instances",
    );
}

fn compare(input: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("belt placement requires Native OrcaSlicer");
        return;
    };
    let work = runner.work_dir();
    let out = write_input(work, input);
    stages::run(
        runner.bin(),
        ARGS.map(str::to_owned),
        stages::Stage::Slice,
        work,
    )
    .unwrap();
    let native = work.join("native");
    fs::rename(&out, &native).unwrap();
    fs::create_dir(&out).unwrap();
    let result = run_ares(work);
    assert!(
        result.status.success(),
        "Ares exited {} in {work:?}",
        result.status
    );
    assert_eq!(result_code(&native), 0, "{work:?}");
    assert_eq!(result_code(&out), 0, "{work:?}");
    let expected = fs::read(native.join("plate_1.gcode")).unwrap();
    let actual = fs::read(out.join("plate_1.gcode")).unwrap();
    golden::compare_ordered_bytes_generator_only(&expected, &actual)
        .unwrap_or_else(|error| panic!("{error}; {work:?}"));
}

fn rejects(input: &[u8], reason: &str) {
    let work = tempfile::Builder::new()
        .prefix("belt-placement-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let out = write_input(&work, input);
    let result = run_ares(&work);
    assert!(
        !result.status.success(),
        "unsupported belt plate accepted in {work:?}"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains(&format!("belt_printer: {reason}")),
        "{work:?}: {stderr}"
    );
    assert!(!out.join("plate_1.gcode").exists(), "{work:?}");
}

fn write_input(work: &Path, input: &[u8]) -> std::path::PathBuf {
    let out = work.join("out");
    fs::create_dir(&out).unwrap();
    fs::write(work.join("input.3mf"), input).unwrap();
    out
}

fn result_code(dir: &Path) -> Value {
    let result: Value =
        serde_json::from_slice(&fs::read(dir.join("result.json")).unwrap()).unwrap();
    result["return_code"].clone()
}
