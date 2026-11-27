use super::{ARGS, CUBE, OrcaRunner, golden, run_ares, stages};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[test]
fn prints_belt_support_and_interface_before_the_object() {
    let Some(runner) = reference() else { return };
    let work = runner.work_dir().join("belt-floor-contact");
    let native = native_slice(&runner, &work, &fixture(Axis::X, "0", &[]));
    witnesses(&native, &work);
    matches_reference(&native, &work);
}

#[test]
fn changing_the_belt_floor_offset_changes_support_motion() {
    let Some(runner) = reference() else { return };
    let unshifted = runner.work_dir().join("belt-support-offset-zero");
    let lowered = runner.work_dir().join("belt-support-offset-lowered");
    let native_zero = native_slice(&runner, &unshifted, &fixture(Axis::Y, "0", &[]));
    let native_offset = native_slice(&runner, &lowered, &fixture(Axis::Y, "-0.8", &[]));
    let (a, b) = (
        witnesses(&native_zero, &unshifted),
        witnesses(&native_offset, &lowered),
    );
    assert!(
        a.base != b.base || a.interface != b.interface,
        "the reference floor offset did not change support extrusion at {lowered:?}"
    );
    matches_reference(&native_zero, &unshifted);
    matches_reference(&native_offset, &lowered);
}

#[test]
fn prints_build_plate_only_support_up_from_the_belt() {
    let Some(runner) = reference() else { return };
    let work = runner.work_dir().join("belt-support-build-plate-only");
    let model = fixture(Axis::X, "0", &[("support_on_build_plate_only", "1")]);
    let native = native_slice(&runner, &work, &model);
    witnesses(&native, &work);
    matches_reference(&native, &work);
}

#[test]
fn keeps_tree_raft_and_brim_with_belt_support_unsupported() {
    for (name, edit) in [
        ("tree", ("support_type", "tree(auto)")),
        ("raft", ("raft_layers", "2")),
        ("brim", ("brim_type", "outer_only")),
    ] {
        let root = crate::artifacts::root_from_env().unwrap();
        let work = tempfile::Builder::new()
            .prefix(&format!("belt-support-{name}-"))
            .tempdir_in(root)
            .unwrap()
            .keep();
        prepare(&work, &fixture(Axis::X, "0", &[("brim_width", "4"), edit]));
        let result = run_ares(&work);
        assert!(!result.status.success(), "{name} was accepted: {work:?}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("belt_printer: support"),
            "wrong rejection for {name}: {work:?}"
        );
        assert!(!work.join("out/plate_1.gcode").exists(), "{work:?}");
    }
}

fn reference() -> Option<OrcaRunner> {
    let runner = OrcaRunner::from_env().unwrap();
    if runner.is_none() {
        crate::native_environment::skipped("normal belt support requires Native OrcaSlicer");
    }
    runner
}

fn prepare(work: &Path, model: &[u8]) -> PathBuf {
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(work.join("input.3mf"), model).unwrap();
    out
}

fn native_slice(runner: &OrcaRunner, work: &Path, model: &[u8]) -> String {
    let out = prepare(work, model);
    stages::run(
        runner.bin(),
        ARGS.map(str::to_owned),
        stages::Stage::Slice,
        work,
    )
    .unwrap();
    fs::rename(&out, work.join("native")).unwrap();
    fs::create_dir(&out).unwrap();
    fs::read_to_string(work.join("native/plate_1.gcode")).unwrap()
}

fn matches_reference(native: &str, work: &Path) {
    let result = run_ares(work);
    assert!(
        result.status.success(),
        "Ares exited {} in {work:?}: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr),
    );
    let actual = fs::read(work.join("out/plate_1.gcode")).unwrap();
    golden::compare_ordered_bytes_generator_only(native.as_bytes(), &actual)
        .unwrap_or_else(|error| panic!("{error}; {work:?}"));
}

#[derive(Default)]
struct Extrusions<'a> {
    base: Vec<(usize, &'a str)>,
    interface: Vec<(usize, &'a str)>,
    first_model_layer: Option<usize>,
}

fn witnesses<'a>(gcode: &'a str, work: &Path) -> Extrusions<'a> {
    let mut result = Extrusions::default();
    let (mut layer, mut role, mut relative, mut extruder) = (0, "", false, 0.0);
    for line in gcode.lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(tag) = line.strip_prefix(";TYPE:") {
            role = tag;
        }
        let motion = line.split_once(';').map_or(line, |(code, _)| code);
        let mut words = motion.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        match command {
            "M82" => relative = false,
            "M83" => relative = true,
            "G1" | "G92" => {
                let (mut e, mut xyz) = (None, false);
                for word in words {
                    if let Some(value) = word.strip_prefix('E') {
                        e = value.parse::<f64>().ok();
                    }
                    xyz |= word.starts_with(['X', 'Y', 'Z']);
                }
                let Some(value) = e else { continue };
                if command == "G92" {
                    extruder = value;
                    continue;
                }
                let delta = if relative { value } else { value - extruder };
                extruder += delta;
                if delta <= 0.0 || !xyz || layer == 0 {
                    continue;
                }
                match role {
                    "Support" => result.base.push((layer, motion)),
                    "Support interface" => result.interface.push((layer, motion)),
                    "" | "Custom" => (),
                    _ => {
                        result.first_model_layer.get_or_insert(layer);
                    }
                }
            }
            _ => (),
        }
    }
    assert!(
        !result.base.is_empty() && !result.interface.is_empty(),
        "expected printed reference support: base={}, interface={} at {work:?}",
        result.base.len(),
        result.interface.len()
    );
    assert!(
        result
            .first_model_layer
            .is_some_and(|model| { result.base[0].0.min(result.interface[0].0) < model }),
        "expected a printed support-only layer before model extrusion at {work:?}"
    );
    result
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

fn fixture(axis: Axis, offset: &str, edits: &[(&str, &str)]) -> Vec<u8> {
    let (axis_name, placement, remap) = match axis {
        Axis::X => ("x", "110 110", ["rev_x", "pos_z", "pos_y"]),
        Axis::Y => ("y", "90 110", ["pos_z", "pos_y", "pos_x"]),
    };
    let mut reader = ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..reader.len() {
        let mut member = reader.by_index(i).unwrap();
        let name = member.name().to_owned();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match name.as_str() {
            "Metadata/project_settings.config" => {
                let mut config: serde_json::Map<String, Value> =
                    serde_json::from_slice(&bytes).unwrap();
                for (key, value) in [
                    ("belt_printer", "1"),
                    ("belt_slice_rotation", axis_name),
                    ("belt_slice_rotation_angle", "45"),
                    ("belt_support_floor_offset", offset),
                    ("enable_arc_fitting", "disabled"),
                    ("enable_support", "1"),
                    ("support_type", "normal(auto)"),
                    ("support_style", "grid"),
                    ("support_on_build_plate_only", "0"),
                    ("support_interface_bottom_layers", "2"),
                    ("support_interface_top_layers", "2"),
                    ("support_threshold_angle", "45"),
                    ("gcode_remap_x", remap[0]),
                    ("gcode_remap_y", remap[1]),
                    ("gcode_remap_z", remap[2]),
                ]
                .into_iter()
                .chain(edits.iter().copied())
                {
                    config.insert(key.to_owned(), json!(value));
                }
                bytes = serde_json::to_vec(&config).unwrap();
            }
            "3D/3dmodel.model" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace(
                        "110 110 0\" printable",
                        &format!("{placement} 0\" printable"),
                    )
                    .into_bytes();
            }
            "3D/Objects/cube10.stl_1.model" => {
                let xml = String::from_utf8(bytes).unwrap();
                let (before, mesh) = xml.split_once("<vertices>").unwrap();
                let (_, after) = mesh.split_once("</triangles>").unwrap();
                bytes = format!("{before}{}{after}", cantilever_mesh(axis)).into_bytes();
            }
            "Metadata/model_settings.config" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace("face_count=\"12\"", "face_count=\"20\"")
                    .into_bytes();
            }
            _ => (),
        }
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn cantilever_mesh(axis: Axis) -> String {
    let boundary = [(0, 0), (8, 0), (8, 16), (-12, 16), (-12, 12), (0, 12)];
    let mut vertices = String::new();
    let mut triangles = String::new();
    for width in [0, 12] {
        for (u, z) in boundary {
            let (x, y) = match axis {
                Axis::X => (width, u),
                Axis::Y => (-u, width),
            };
            vertices.push_str(&format!("<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>"));
        }
    }
    let mut triangle =
        |a, b, c| triangles.push_str(&format!("<triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>"));
    for (a, b, c) in [(0, 1, 5), (1, 2, 5), (2, 3, 4), (2, 4, 5)] {
        triangle(a, c, b);
        triangle(a + 6, b + 6, c + 6);
    }
    for a in 0..6 {
        let b = (a + 1) % 6;
        triangle(a, b, b + 6);
        triangle(a, b + 6, a + 6);
    }
    format!("<vertices>{vertices}</vertices><triangles>{triangles}</triangles>")
}
