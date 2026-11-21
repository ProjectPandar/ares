//! An STL loaded with selected presets onto an empty plate is centered on the bed and dropped
//! (`Plater::priv::load_model_objects`, Plater.cpp:7056-7058, 7111-7112, 7149-7160); plater
//! edits then move it. Native slices the cube pre-positioned where the session puts it, with
//! the same presets (`--load-settings`, `--arrange 0`), as the center-placement case does.

use serde_json::{Value, json};

use super::super::{OrcaRunner, stages};
use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, PlaterSession, ProfileSliceRequest,
    SlicerContext,
};

const MACHINE: &str = "LH Stinger 0.4 nozzle";

#[path = "plater_stl_session/fill_geometry.rs"]
mod fill_geometry;
#[path = "plater_stl_session/honeycomb_geometry.rs"]
mod honeycomb_geometry;
#[path = "klipper_time_estimator.rs"]
mod klipper_time_estimator;
#[path = "plater_stl_session/plane_path_smoothing.rs"]
mod plane_path_smoothing;

struct Presets {
    catalog: String,
    selection: String,
}

/// Writes the flattened presets for the Native CLI; the session gets the raw catalog.
fn presets(runner: &OrcaRunner) -> Presets {
    let profiles_root = super::super::repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let process = profiles.compatible_process(MACHINE).unwrap();
    let filament = profiles.compatible_filament(MACHINE).unwrap();
    for (kind, mut preset) in [
        ("machine", profiles.machine(MACHINE).unwrap()),
        ("process", profiles.process(&process).unwrap()),
        ("filament", profiles.filament(&filament).unwrap()),
    ] {
        preset.insert("type".to_owned(), json!(kind));
        preset.insert("from".to_owned(), json!("system"));
        preset.insert("instantiation".to_owned(), json!("true"));
        super::write_json(
            &runner.work.join(format!("stl-session-{kind}.json")),
            &Value::Object(preset),
        );
    }
    Presets {
        catalog: json!({
            "vendors": [
                super::vendor(&profiles_root, "OrcaFilamentLibrary"),
                super::vendor(&profiles_root, "LH"),
            ],
            "imports": [],
        })
        .to_string(),
        selection: json!({"machine": MACHINE, "process": process, "filaments": [filament]})
            .to_string(),
    }
}

/// cube10.stl with every vertex moved by `shift`.
fn cube_at(shift: [f32; 3]) -> Vec<u8> {
    let mut bytes =
        std::fs::read(super::super::repo_root().join("tests/parity/cube10.stl")).unwrap();
    for facet in bytes[84..].as_chunks_mut::<50>().0 {
        for vertex in facet[12..48].as_chunks_mut::<12>().0 {
            for (coordinate, shift) in vertex.as_chunks_mut::<4>().0.iter_mut().zip(shift) {
                *coordinate = (f32::from_le_bytes(*coordinate) + shift).to_le_bytes();
            }
        }
    }
    bytes
}

/// Native slicing `stl` saved as cube10.stl, the name the session's object and labels carry.
fn native(runner: &OrcaRunner, label: &str, stl: &[u8]) -> Vec<u8> {
    let directory = runner.work.join(label);
    let model = directory.join("cube10.stl");
    let output = directory.join("reference");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(&model, stl).unwrap();
    let file = |kind: &str| runner.work.join(format!("stl-session-{kind}.json"));
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            super::super::join_paths([file("machine"), file("process")].iter()),
            "--load-filaments".to_owned(),
            file("filament").to_str().unwrap().to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            model.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    std::fs::read(output.join("plate_1.gcode")).unwrap()
}

fn request<'a>(presets: &'a Presets, overrides: &'a ConfigOverrides) -> ProfileSliceRequest<'a> {
    ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &presets.catalog,
        selection_json: &presets.selection,
        plate: Some(1),
        overrides,
        placement: Placement::Center,
    }
}

fn slice(session: &PlaterSession, presets: &Presets) -> Vec<u8> {
    let overrides = ConfigOverrides::default();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_plater_with_profiles(
            session,
            request(presets, &overrides),
            false,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            SlicerContext::new(0),
        ))
        .unwrap()
        .gcode
}

#[test]
fn plater_stl_session_centers_moves_and_slices_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let presets = presets(&runner);
    let native_presets = super::native_user_presets::resolved(
        &runner.work,
        &super::super::repo_root().join("OrcaSlicer/resources/profiles"),
        &serde_json::from_str(&presets.catalog).unwrap(),
        &serde_json::from_str(&presets.selection).unwrap(),
    );
    let native = |label: &str, stl: &[u8]| {
        let directory = runner.work.join(label);
        std::fs::create_dir_all(&directory).unwrap();
        let model = directory.join("cube10.stl");
        std::fs::write(&model, stl).unwrap();
        super::native_user_presets::slice_stl(
            &runner,
            &native_presets,
            &model,
            &format!("{label}/reference"),
            &json!({}),
        )
    };
    let overrides = ConfigOverrides::default();
    let stl = std::fs::read(super::super::repo_root().join("tests/parity/cube10.stl")).unwrap();
    let mut session = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(PlaterSession::load_with_profiles(
            stl,
            request(&presets, &overrides),
        ))
        .unwrap();
    // Loaded onto the empty LH Stinger plate: bbox center at the bed center (118, 138), grounded.
    let centered = native("stl-session-centered", &cube_at([118.0, 138.0, 0.0]));
    let actual = slice(&session, &presets);
    std::fs::write(runner.work.join("stl-session-centered/ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&centered, &actual).unwrap();
    // A raised move drops back onto the bed (GLCanvas3D::do_move).
    session
        .translate_instance(0, 0, [20.0, -15.0, 3.0])
        .unwrap();
    let moved = native("stl-session-moved", &cube_at([138.0, 123.0, 0.0]));
    let actual = slice(&session, &presets);
    std::fs::write(runner.work.join("stl-session-moved/ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&moved, &actual).unwrap();
}
