use super::{ARGS, OrcaRunner, golden, run_ares, stages};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const CANTILEVER: &[u8] =
    include_bytes!("../../../../../tests/parity/belt-tree-support/organic-cantilever.3mf");

#[tokio::test]
async fn prints_organic_branches_on_a_translated_belt() {
    let Some(runner) = reference() else { return };
    let work = runner.work_dir().join("organic-belt");
    let native = native_slice(&runner, &work, CANTILEVER);
    assert!(!support_moves(&native).is_empty(), "{work:?}");
    matches_reference(&native, &work, CANTILEVER).await;
}

#[tokio::test]
async fn raising_the_belt_floor_changes_organic_support_motion() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let work = root.join("organic-raised-floor");
    let raised = project(&[("belt_support_floor_offset", "1")], false);
    let native = native_slice(&runner, &work, &raised);
    let zero = native_slice(&runner, &root.join("organic-floor-zero"), CANTILEVER);
    let (raised_moves, zero_moves) = (support_moves(&native), support_moves(&zero));
    assert!(
        !raised_moves.is_empty() && !zero_moves.is_empty(),
        "{root:?}"
    );
    assert!(
        raised_moves != zero_moves,
        "the Native floor offset must change support extrusion moves at {root:?}"
    );
    matches_reference(&native, &work, &raised).await;
}

#[tokio::test]
async fn default_tree_style_prints_organic_branches_on_a_y_tilted_belt() {
    let Some(runner) = reference() else { return };
    let work = runner.work_dir().join("organic-belt-y-default");
    let model = project(
        &[
            ("support_style", "default"),
            ("belt_slice_rotation", "y"),
            ("gcode_remap_x", "pos_z"),
            ("gcode_remap_y", "pos_y"),
            ("gcode_remap_z", "pos_x"),
        ],
        true,
    );
    let native = native_slice(&runner, &work, &model);
    assert!(!support_moves(&native).is_empty(), "{work:?}");
    matches_reference(&native, &work, &model).await;
}

#[test]
fn defers_other_tree_styles_and_tree_on_model_on_belts() {
    let root = crate::artifacts::root_from_env().unwrap();
    for (key, value) in [
        ("support_style", "tree_slim"),
        ("support_style", "tree_strong"),
        ("support_style", "tree_hybrid"),
        ("support_on_build_plate_only", "0"),
    ] {
        let work = tempfile::Builder::new()
            .prefix("unsupported-belt-tree-")
            .tempdir_in(&root)
            .unwrap()
            .keep();
        prepare(&work, &project(&[(key, value)], false));
        let result = run_ares(&work);
        assert!(!result.status.success(), "{key}={value} at {work:?}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("belt_printer: support"),
            "wrong rejection for {key}={value} at {work:?}"
        );
        assert!(!work.join("out/plate_1.gcode").exists(), "{work:?}");
    }
}

fn reference() -> Option<OrcaRunner> {
    let runner = OrcaRunner::from_env().unwrap();
    if runner.is_none() {
        crate::native_environment::skipped("organic belt support requires Native OrcaSlicer");
    }
    runner
}

fn prepare(work: &Path, project: &[u8]) {
    fs::create_dir_all(work.join("out")).unwrap();
    fs::write(work.join("input.3mf"), project).unwrap();
}

fn native_slice(runner: &OrcaRunner, work: &Path, project: &[u8]) -> String {
    prepare(work, project);
    stages::run(
        runner.bin(),
        ARGS.map(str::to_owned),
        stages::Stage::Slice,
        work,
    )
    .unwrap();
    let out = work.join("out");
    let native = work.join("native");
    fs::rename(&out, &native).unwrap();
    fs::create_dir(out).unwrap();
    fs::read_to_string(native.join("plate_1.gcode")).unwrap()
}

async fn matches_reference(native: &str, work: &Path, project: &[u8]) {
    let result = run_ares(work);
    assert!(
        result.status.success(),
        "Ares exited {} in {work:?}: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr),
    );
    let cli = fs::read(work.join("out/plate_1.gcode")).unwrap();
    golden::compare_ordered_bytes_generator_only(native.as_bytes(), &cli)
        .unwrap_or_else(|error| panic!("CLI: {error}; {work:?}"));
    let api = crate::runner::application::slice_project_plate_validated(
        project,
        GenerationMetadata::deterministic(2026, 1, 1, 0, 0, 0),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_or_else(|error| panic!("public API: {error:?}; {work:?}"));
    fs::write(work.join("ares.api.gcode"), &api).unwrap();
    golden::compare_ordered_bytes_generator_only(native.as_bytes(), &api)
        .unwrap_or_else(|error| panic!("public API: {error}; {work:?}"));
}

fn support_moves(gcode: &str) -> Vec<&str> {
    let mut base = false;
    let mut relative = false;
    gcode
        .lines()
        .filter_map(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                base = role == "Support";
            }
            let motion = line.split(';').next().unwrap();
            let mut words = motion.split_ascii_whitespace();
            match words.next()? {
                "M82" => relative = false,
                "M83" => relative = true,
                "G0" | "G1" if base && relative => {
                    let (mut xyz, mut extrusion) = (false, false);
                    for word in words {
                        xyz |= word.starts_with(['X', 'Y', 'Z']);
                        extrusion |= word
                            .strip_prefix('E')
                            .and_then(|e| e.parse::<f64>().ok())
                            .is_some_and(|e| e > 0.0);
                    }
                    if xyz && extrusion {
                        return Some(motion);
                    }
                }
                _ => {}
            }
            None
        })
        .collect()
}

fn project(edits: &[(&str, &str)], turn_about_z: bool) -> Vec<u8> {
    let mut reader = ZipArchive::new(Cursor::new(CANTILEVER)).unwrap();
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
                config.extend(
                    edits
                        .iter()
                        .map(|(key, value)| ((*key).to_owned(), json!(value))),
                );
                bytes = serde_json::to_vec(&config).unwrap();
            }
            "3D/3dmodel.model" if turn_about_z => {
                let model = String::from_utf8(bytes).unwrap();
                let placement = "1 0 0 0 1 0 0 0 1 110 146 0";
                assert_eq!(model.matches(placement).count(), 1);
                bytes = model
                    .replace(placement, "0 1 0 -1 0 0 0 0 1 74 110 0")
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
