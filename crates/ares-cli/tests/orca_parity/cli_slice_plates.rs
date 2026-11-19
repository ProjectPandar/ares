#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

use crate::{
    golden::compare_ordered_bytes_generator_only,
    native_wall_clock::result_comparison_bytes,
    runner::{OrcaRunner, application::cli_command, stages},
};

const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/custom-gcode-overview/two-plates.3mf");

#[test]
fn cli_slice_all_plates_exports_every_plate_in_order() {
    verify("0", PROJECT, true, 0, &["plate_1.gcode", "plate_2.gcode"]);
}

#[test]
fn cli_slice_positive_keeps_the_native_plate_filename() {
    verify("2", PROJECT, true, 0, &["plate_2.gcode"]);
}

#[test]
fn cli_slice_without_outputdir_keeps_only_the_report() {
    verify("0", PROJECT, false, 0, &[]);
}

#[test]
fn cli_slice_rejects_signed_and_out_of_range_selections() {
    for selection in ["-1", "3"] {
        verify(selection, PROJECT, true, 254, &[]);
    }
}

#[test]
fn cli_slice_refuses_to_export_a_plate_with_critical_empty_layers() {
    let input = include_bytes!("../../../../tests/parity/mesh-edge-pairing/odd-rotated.3mf");
    verify("0", input, true, 156, &[]);
}

#[test]
fn cli_slice_all_plates_preflights_later_plates_before_export() {
    let mut source = zip::ZipArchive::new(Cursor::new(PROJECT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.name() == "3D/3dmodel.model" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            let old = " 374 110 5\" printable=\"1\"";
            assert_eq!(xml.matches(old).count(), 1);
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(xml.replace(old, " 374 110 5\" printable=\"0\"").as_bytes())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    let changed = output.finish().unwrap().into_inner();
    verify("0", &changed, true, 206, &[]);
}

fn verify(selection: &str, input: &[u8], outputdir: bool, exit: i32, names: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-slice-plates-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = root.join("project.3mf");
    fs::write(&input_path, input).unwrap();
    let mut references = Vec::new();
    for name in ["native", "native-replay"] {
        let work = root.join(name);
        create_work(&work, outputdir && exit != 0);
        let args = args(selection, &input_path, outputdir);
        let result = stages::run_live(runner.bin(), args, stages::Stage::Slice, &work);
        let code = result.map_or_else(|error| error.captured.unwrap().code, |()| Some(0));
        assert_eq!(code, Some(exit), "{work:?}");
        references.push(outputs(&work, outputdir, names));
    }
    compare(&references[0], &references[1], true, &root);
    let work = root.join("ares");
    create_work(&work, outputdir && exit != 0);
    let actual = cli_command()
        .current_dir(&work)
        .arg("slice")
        .args(args(selection, &input_path, outputdir))
        .output()
        .unwrap();
    fs::write(work.join("stdout"), &actual.stdout).unwrap();
    fs::write(work.join("stderr"), &actual.stderr).unwrap();
    assert_eq!(
        actual.status.code(),
        Some(exit),
        "{work:?}: {}",
        String::from_utf8_lossy(&actual.stderr),
    );
    let actual = outputs(&work, outputdir, names);
    compare(&references[0], &actual, false, &root);
    if !names.is_empty() {
        compare_api(input, selection.parse().unwrap(), names, &actual.0);
    }
}

fn compare_api(input: &[u8], selection: i32, names: &[&str], cli: &[Vec<u8>]) {
    let mut published = Vec::new();
    ares_core::slice_cli_project_plates(
        input,
        ares_core::CliPlatesRequest {
            selection,
            filename: "project.3mf",
            settings: ares_core::CliPlatesSettings::Embedded,
            overrides: &ares_core::ConfigOverrides::default(),
            orient: false,
            upward: ares_core::CliUpwardMachine {
                source: ares_core::UpwardMachineSource::from_project(input).unwrap(),
                input: ares_core::UpwardMachineInput::FetchOriginal(None),
                report: &mut Vec::new(),
            },
        },
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
        |plate, gcode, _| {
            let index = published.len();
            let name = format!("plate_{plate}.gcode");
            assert_eq!(name, names[index]);
            assert_eq!(gcode, &cli[index], "raw API/CLI G-code: {name}");
            published.push(name);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(published, names);
}

fn create_work(work: &Path, existing_output_dir: bool) {
    fs::create_dir(work).unwrap();
    if existing_output_dir {
        fs::create_dir(work.join("out")).unwrap();
    }
}

fn args(selection: &str, input: &Path, outputdir: bool) -> Vec<String> {
    let mut args = vec!["--slice".to_owned(), selection.to_owned()];
    if outputdir {
        args.extend(["--outputdir".to_owned(), "out".to_owned()]);
    }
    args.push(input.to_str().unwrap().to_owned());
    args
}

fn outputs(work: &Path, outputdir: bool, names: &[&str]) -> (Vec<Vec<u8>>, Vec<u8>) {
    let directory = if outputdir {
        work.join("out")
    } else {
        work.to_owned()
    };
    let mut found = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gcode"))
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    found.sort_unstable();
    assert_eq!(found, names, "{directory:?}");
    (
        names
            .iter()
            .map(|name| fs::read(directory.join(name)).unwrap())
            .collect(),
        fs::read(directory.join("result.json")).unwrap(),
    )
}

fn compare(
    expected: &(Vec<Vec<u8>>, Vec<u8>),
    actual: &(Vec<Vec<u8>>, Vec<u8>),
    native_replay: bool,
    root: &Path,
) {
    for (expected, actual) in expected.0.iter().zip(&actual.0) {
        if native_replay {
            use crate::golden::{GeneratorKind, normalize_one_generator_line};
            assert_eq!(
                normalize_one_generator_line(expected, GeneratorKind::Orca).unwrap(),
                normalize_one_generator_line(actual, GeneratorKind::Orca).unwrap(),
                "Native replay: {root:?}"
            );
        } else {
            compare_ordered_bytes_generator_only(expected, actual)
                .unwrap_or_else(|error| panic!("{root:?}: {error}"));
        }
    }
    assert_eq!(
        result_comparison_bytes(&expected.1),
        result_comparison_bytes(&actual.1),
        "result.json: {root:?}"
    );
}
