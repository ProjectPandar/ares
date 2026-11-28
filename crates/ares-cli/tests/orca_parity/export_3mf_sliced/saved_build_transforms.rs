use std::{fs, path::Path};

use quick_xml::{Reader, events::Event};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{SETTINGS, STL, comparator, invoke, write_inputs};
use crate::OrcaRunner;

const MODEL: &str = "3D/3dmodel.model";
const PROJECT: &str = "saved.3mf";
const EXPORT: &str = "export.3mf";

#[test]
fn saved_sliced_archive_preserves_tilted_build_instance() {
    check("1 0 0 0 0 1 0 -1 0 128 128 5", true);
}

#[test]
fn saved_sliced_archive_preserves_planar_build_rotation() {
    check("0.8 0.6 0 -0.6 0.8 0 0 0 1 128 128 1", false);
}

#[test]
fn saved_sliced_archive_preserves_scaled_reflection() {
    check("-2 0 0 0 2 0 0 0 2 128 128 2", false);
}

#[test]
fn saved_sliced_archive_preserves_xy_build_shear() {
    check("1 0 0 0.5 1 0 0 0 1 128 128 1", false);
}

fn check(transform: &str, no_brim: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(source.join("out")).unwrap();
    write_inputs(&source);
    if no_brim {
        // Keep the narrow tilted footprint independent of automatic brim sizing.
        let process = source.join("process.json");
        let mut settings: serde_json::Value =
            serde_json::from_slice(&fs::read(&process).unwrap()).unwrap();
        settings["brim_type"] = serde_json::json!("no_brim");
        fs::write(process, serde_json::to_vec(&settings).unwrap()).unwrap();
    }
    fs::write(source.join(STL), prism_stl()).unwrap();
    let source_argv = [SETTINGS.as_slice(), &["--export-3mf", PROJECT, STL]].concat();
    assert_eq!(invoke(runner.bin(), &source, &source_argv), Some(0));

    let project = rewrite_project(&fs::read(source.join(PROJECT)).unwrap(), |name, xml| {
        if name != MODEL {
            return xml.to_owned();
        }
        let item = xml.lines().find(|line| line.contains("<item ")).unwrap();
        let (_, attr) = item.split_once("transform=\"").unwrap();
        let (old, _) = attr.split_once('"').unwrap();
        xml.replace(item, &item.replace(old, transform))
    });
    let export = |bin: &Path, name: &str| {
        let cwd = root.join(name).join("cwd");
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join(PROJECT), &project).unwrap();
        let argv = ["--slice", "1", "--export-3mf", EXPORT, PROJECT];
        assert_eq!(invoke(bin, &cwd, &argv), Some(0), "export in {cwd:?}");
        let outputs = fs::read_dir(&cwd)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .filter(|name| name != PROJECT && matches!(name.to_str(), Some(name) if name.ends_with(".3mf") || name.ends_with(".gcode")))
            .collect::<Vec<_>>();
        assert_eq!(outputs, [EXPORT]);
        assert_eq!(fs::read_dir(cwd.join("out")).unwrap().count(), 0);

        let members = comparator::read_members(&cwd.join(EXPORT));
        let model = members.iter().find(|entry| entry.name == MODEL).unwrap();
        assert_build_item(&model.bytes, transform);
        members
    };
    let native = export(runner.bin(), "native");
    let ares = export(Path::new(env!("CARGO_BIN_EXE_ares")), "ares");
    comparator::compare_members(native, ares);
}

fn assert_build_item(model: &[u8], expected: &str) {
    let matrix = |value: &str| {
        value
            .split_ascii_whitespace()
            .map(|entry| entry.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    };
    let mut reader = Reader::from_reader(model);
    let mut count = 0;
    loop {
        match reader.read_event().unwrap() {
            Event::Empty(element) if element.local_name().as_ref() == b"item" => {
                let get = |key| {
                    element
                        .try_get_attribute(key)
                        .unwrap()
                        .unwrap()
                        .decode_and_unescape_value(reader.decoder())
                        .unwrap()
                        .into_owned()
                };
                assert_eq!(get("objectid"), "2");
                assert_eq!(get("printable"), "1");
                assert_eq!(matrix(&get("transform")), matrix(expected));
                count += 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(count, 1);
}

fn prism_stl() -> Vec<u8> {
    let outline = [
        [-15.0_f32, -5.0_f32],
        [15.0, -5.0],
        [15.0, 0.0],
        [10.0, 5.0],
        [-15.0, 5.0],
    ];
    let n = outline.len();
    let faces = (1..n - 1)
        .flat_map(|i| [[0, i + 1, i], [n, n + i, n + i + 1]])
        .chain((0..n).flat_map(|i| {
            let next = (i + 1) % n;
            [[i, next, next + n], [i, next + n, i + n]]
        }));
    let mut bytes = vec![0; 80];
    bytes.extend(16_u32.to_le_bytes());
    for face in faces {
        bytes.extend([0; 12]);
        for vertex in face {
            let [x, y] = outline[vertex % n];
            let z: f32 = if vertex < n { -1.0 } else { 1.0 };
            for coordinate in [x, y, z] {
                bytes.extend(coordinate.to_le_bytes());
            }
        }
        bytes.extend([0; 2]);
    }
    bytes
}
