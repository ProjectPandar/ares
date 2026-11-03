//! `Print::apply`'s print instances are the `ModelInstance::is_printable()` ones
//! (`PrintApply.cpp:201` at v2.5.0-beta `345efec64`; `Model.hpp:1514`): printable and inside the
//! print volume, the state `Model::update_print_volume_state` stores (`Model.cpp:839-852`). See
//! `docs/architecture/print-apply-printable-instances.md`.

use std::fs;
use std::io::{Cursor, Read, Write};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::runner::{OrcaRunner, stages};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
const CUBES: &[u8] =
    include_bytes!("../../../../tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf");

/// The Ender-3 cube at (110, 110) plus a second instance of it at x = 400, fully outside the
/// 220 mm bed, both on plate 1: Native prints the inside cube only.
#[test]
fn print_apply_leaves_a_fully_outside_instance_out_like_native() {
    assert_plate_one_like_native("print-volume-printable-", &with_outside_copy());
}

/// The selected-plate import keeps every instance of an object the plate lists
/// (`bbs_3mf.cpp:2215-2221`; see `docs/architecture/selected-plate-object-import.md`): with object
/// 2's second pair moved to a new plate 2 while the instance still sits on plate 1's area, Native
/// `--slice 1` prints all three cubes.
#[test]
fn selected_plate_import_keeps_every_instance_of_a_listed_object_like_native() {
    assert_plate_one_like_native("selected-plate-object-", &with_pair_on_plate_two());
}

/// Native `--slice 1` of `input` and Ares's plate-1 slice, compared as ordered bytes.
fn assert_plate_one_like_native(prefix: &str, input: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = fs::read(output.join("plate_1.gcode")).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate(
            input,
            GenerationMetadata::default(),
            Some(1),
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&native, &actual).unwrap();
}

/// The cube project with object 2's second build item at (400, 110) and its plate-1
/// `model_instance` (instance 1, identify id 15).
fn with_outside_copy() -> Vec<u8> {
    patched(CUBE, |name, text| {
        match name {
        "3D/3dmodel.model" => text.replacen(
            "</build>",
            "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 400 110 5\" printable=\"1\"/></build>",
            1,
        ),
        "Metadata/model_settings.config" => text.replacen(
            "  </plate>",
            "    <model_instance>\n      <metadata key=\"object_id\" value=\"2\"/>\n      <metadata key=\"instance_id\" value=\"1\"/>\n      <metadata key=\"identify_id\" value=\"15\"/>\n    </model_instance>\n  </plate>",
            1,
        ),
        _ => text,
    }
    })
}

/// `combined-brim.3mf` with object 2's second pair (instance 1, identify id 15) moved from plate 1
/// to a new plate 2; the instance itself stays at (125, 110) on plate 1's area.
fn with_pair_on_plate_two() -> Vec<u8> {
    const PAIR: &str = "<model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"1\"/><metadata key=\"identify_id\" value=\"15\"/></model_instance>";
    patched(CUBES, |name, text| match name {
        "Metadata/model_settings.config" => {
            assert!(text.contains(PAIR), "fixture lists object 2's second pair");
            text.replacen(PAIR, "", 1).replacen(
                "</plate>",
                &format!("</plate>\n  <plate>\n    <metadata key=\"plater_id\" value=\"2\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n  {PAIR}</plate>"),
                1,
            )
        }
        "Metadata/filament_sequence.json" => text.replacen(
            "}}",
            "},\"plate_2\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}",
            1,
        ),
        _ => text,
    })
}

/// `input` with each text entry passed through `patch`.
fn patched(input: &[u8], patch: impl Fn(&str, String) -> String) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let bytes = match String::from_utf8(bytes) {
            Ok(text) => patch(file.name(), text).into_bytes(),
            Err(error) => error.into_bytes(),
        };
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
