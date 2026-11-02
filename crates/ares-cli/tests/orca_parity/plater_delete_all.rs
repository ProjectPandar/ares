//! The Edit menu's "Delete All" on the plater session (`Plater::priv::delete_all_objects_from_model`,
//! `Plater.cpp:10793-10821` at v2.5.0-beta `345efec64`; see
//! `docs/architecture/web-plater-delete-all.md`): the model loses every object, every plate its
//! instances (`PartPlateList::clear`, `PartPlate.cpp:6454-6499`, keeps the plates, the current plate
//! and each plate's lock, `PartPlate::clear`, `:4173-4184`), and `model.plates_custom_gcodes` is
//! cleared. Native slicing the project the GUI then exports fails to read the empty model
//! (`Model.cpp:500-501`), as the session's slice must.

use std::io::{Cursor, Read, Write};

use ares_core::SliceError;

use super::{STRIDE, TWO_PLATES, export, load};
use crate::runner::{OrcaRunner, stages};

const MESSAGE: &str = "The supplied file couldn't be read because it's empty.";

#[test]
fn plater_delete_all_empties_the_plates_and_fails_to_slice_like_native() {
    let mut session = load(TWO_PLATES);
    session.select_plate(1).unwrap();
    session.lock_plate(1, true).unwrap();
    // Below the grid, right of and below the unprintable plate's origin (the next grid cell,
    // PartPlate.cpp:3942-3949), which then holds it (is_left_top_of, :2690-2708).
    session
        .translate_instance(0, 0, [0.0, -2.0 * STRIDE, 0.0])
        .unwrap();
    assert_eq!(session.unprintable_instances(), vec![(0, 0)]);
    assert!(!session.project().plates_custom_gcodes().is_empty());

    session.delete_all_objects().unwrap();
    assert!(session.project().objects().is_empty());
    let plates = session.plates();
    assert_eq!(plates.len(), 2);
    assert!(plates.iter().all(|plate| plate.instances.is_empty()));
    assert!(session.unprintable_instances().is_empty());
    assert_eq!(session.current_plate(), 1);
    assert!(session.is_plate_locked(1).unwrap());
    assert!(!session.is_plate_locked(0).unwrap());
    assert!(session.project().plates_custom_gcodes().is_empty());

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_plater_plate(
            &session,
            ares_core::PlaterSlice {
                plate: Some(1),
                overrides: &ares_core::ConfigOverrides::default(),
                validate_print: false,
            },
            ares_core::GenerationMetadata::default(),
            ares_core::SlicerContext::new(0),
        ));
    assert!(matches!(actual, Err(SliceError::NoSuitableObjects(text)) if text == MESSAGE));
    assert_native_read_error(&session);
}

#[test]
fn plater_delete_all_refuses_an_empty_model() {
    let mut session = load(TWO_PLATES);
    session.delete_all_objects().unwrap();
    assert!(matches!(
        session.delete_all_objects(),
        Err(SliceError::InvalidInput(_))
    ));
}

/// The GUI export of the emptied session: no objects, and no `custom_gcode_per_layer.xml`, which
/// the exporter writes only with custom G-code (`bbs_3mf.cpp:9079-9117`).
fn exported(session: &ares_core::PlaterSession) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(export(session))).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        if file.name() == "Metadata/custom_gcode_per_layer.xml" {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

/// Native `--slice 1` of the export fails with the same model read error.
fn assert_native_read_error(session: &ares_core::PlaterSession) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("plater-delete-all-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    std::fs::write(&path, exported(session)).unwrap();
    std::fs::create_dir(&output).unwrap();
    let error = stages::run(
        runner.bin(),
        [
            "--slice",
            "1",
            "--outputdir",
            output.to_str().unwrap(),
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    let captured = error.captured.as_ref().unwrap();
    assert_eq!(
        captured.stderr,
        format!("{}: {MESSAGE}\n", path.display()).as_bytes()
    );
}
