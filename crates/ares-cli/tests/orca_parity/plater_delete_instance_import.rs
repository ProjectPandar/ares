//! The project the GUI exports after a middle instance delete loads like Native's import: a plate
//! entry with `identify_id` 0 and a stale instance index (`bbs_3mf.cpp:8580-8600` at v2.5.0-beta
//! `345efec64`) lists its object on the plate (`:2215-2221`) and labels no instance (`:2466-2500`).
//! See `docs/architecture/plate-dangling-entry-import.md`.

use ares_core::{ConfigOverrides, GenerationMetadata, PlaterSession, SlicerContext};

use super::{STRIDE, export, load};
use crate::runner::OrcaRunner;

/// Two 10 mm cubes on an Ender-3: object 2 with two instances, object 3 with one.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

#[test]
fn plater_delete_instance_export_loads_and_slices_like_native() {
    let mut session = load(CUBES);
    session.delete_instance(0, 0).unwrap();
    assert_export_like_native("delete-instance-import-same-plate", &session, 1);
}

#[test]
fn plater_cross_plate_delete_export_loads_and_slices_like_native() {
    let mut session = load(CUBES);
    session.add_plate().unwrap();
    session
        .translate_instance(0, 1, [STRIDE, 0.0, 0.0])
        .unwrap();
    session.delete_instance(0, 0).unwrap();
    assert_export_like_native("delete-instance-import-cross-plate", &session, 1);
    assert_export_like_native("delete-instance-import-cross-plate-2", &session, 2);
}

/// Native `--slice <plate>` of the session's export first, then Ares loading the same export as a
/// project and slicing that plate.
fn assert_export_like_native(label: &str, session: &PlaterSession, plate: u32) {
    let exported = export(session);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = runner
        .fixture_case(&format!("plater/{label}"), exported.clone())
        .unwrap();
    let native = runner.slice_selected_plate_case(&case, plate).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_direct_project_plate(
            exported.as_slice(),
            GenerationMetadata::default(),
            Some(plate),
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&native.reference, &actual).unwrap();
}
