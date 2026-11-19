#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::runner::{OrcaRunner, stages};

pub(super) const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/orca_cli_inspection_paint.3mf");

#[test]
fn info_reports_open_edges_without_slicing() {
    compare(&["--info"], Some(("open.stl", open_box())), 0);
}

#[test]
fn info_reports_each_objects_model_parts() {
    compare(&["--info"], Some(("paint.3mf", PROJECT.to_vec())), 0);
}

#[test]
fn paint_counts_each_layer_on_multiple_objects_and_volumes() {
    compare(
        &["--inspect-paint"],
        Some(("paint.3mf", PROJECT.to_vec())),
        0,
    );
}

#[test]
fn paint_reports_empty_layers_on_stl() {
    compare(&["--inspect-paint"], Some(("open.stl", open_box())), 0);
}

#[test]
fn paint_inspection_rejects_slicing_before_loading() {
    compare(&["--inspect-paint", "--slice", "0"], None, 254);
}

#[test]
fn paint_inspection_rejects_an_export_action() {
    compare(&["--export-stl", "--inspect-paint"], None, 254);
}

#[test]
fn paint_inspection_requires_input() {
    compare(&["--inspect-paint"], None, 254);
}

#[test]
fn info_succeeds_without_input() {
    compare(&["--info"], None, 0);
}

pub(super) fn compare(flags: &[&str], input: Option<(&str, Vec<u8>)>, exit: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-model-inspection-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = input.map(|(filename, bytes)| {
        let path = root.join(filename);
        fs::write(&path, bytes).unwrap();
        path.into_os_string().into_string().unwrap()
    });
    let native = run_native(&runner, &root.join("native"), flags, input.as_deref());
    assert_eq!(native.0, Some(exit), "Native: {root:?}");
    assert_eq!(
        native,
        run_native(
            &runner,
            &root.join("native-replay"),
            flags,
            input.as_deref()
        ),
        "Native replay: {root:?}"
    );
    let work = root.join("ares");
    fs::create_dir(&work).unwrap();
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(&work)
        .arg("slice")
        .args(flags)
        .args(["--outputdir", "."])
        .args(input.iter())
        .output()
        .unwrap();
    fs::write(root.join("ares.stdout"), &ares.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &ares.stderr).unwrap();
    assert_eq!(ares.status.code(), native.0, "exit: {root:?}");
    assert!(ares.stdout == native.1, "stdout: {root:?}");
    assert!(ares.stderr == native.2, "stderr: {root:?}");
    assert_eq!(inventory(&work), native.3, "outputs: {root:?}");
}

pub(super) fn run_native(
    runner: &OrcaRunner,
    work: &Path,
    flags: &[&str],
    input: Option<&str>,
) -> (Option<i32>, Vec<u8>, Vec<u8>, Vec<String>) {
    fs::create_dir_all(work.join("outputs")).unwrap();
    let args = ["--debug", "0"]
        .into_iter()
        .chain(flags.iter().copied())
        .chain(["--outputdir", "outputs"])
        .chain(input)
        .map(str::to_owned);
    let result = stages::run_live(runner.bin(), args, stages::Stage::Slice, work);
    let exit = result.map_or_else(|error| error.captured.unwrap().code, |()| Some(0));
    let files = inventory(&work.join("outputs"));
    let capture = inventory(work)
        .into_iter()
        .find(|name| name.starts_with("command-"))
        .map(|name| work.join(name))
        .unwrap();
    assert_eq!(files, ["result.json"], "unexpected Native output: {work:?}");
    (
        exit,
        fs::read(capture.join("stdout.bin")).unwrap(),
        fs::read(capture.join("stderr.bin")).unwrap(),
        files,
    )
}

pub(super) fn inventory(work: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(work)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort_unstable();
    names
}

pub(super) fn open_box() -> Vec<u8> {
    use std::fmt::Write;
    let vertices = [
        [0, 0, 0],
        [8, 0, 0],
        [8, 12, 0],
        [0, 12, 0],
        [0, 0, 16],
        [8, 0, 16],
        [8, 12, 16],
        [0, 12, 16],
    ];
    let mut text = String::from("solid open\n");
    for face in [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
    ] {
        text.push_str("facet normal 0 0 0\n outer loop\n");
        for [x, y, z] in face.map(|index| vertices[index]) {
            writeln!(text, "  vertex {x} {y} {z}").unwrap();
        }
        text.push_str(" endloop\nendfacet\n");
    }
    text.push_str("endsolid open\n");
    text.into_bytes()
}
