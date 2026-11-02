//! `Print::apply`'s print instances are the `ModelInstance::is_printable()` ones
//! (`PrintApply.cpp:201` at v2.5.0-beta `345efec64`; `Model.hpp:1514`): printable and inside the
//! print volume, the state `Model::update_print_volume_state` stores (`Model.cpp:839-852`). See
//! `docs/architecture/print-apply-printable-instances.md`.

use std::fs;
use std::io::{Cursor, Read, Write};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::runner::{OrcaRunner, stages};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");

/// The Ender-3 cube at (110, 110) plus a second instance of it at x = 400, fully outside the
/// 220 mm bed, both on plate 1: Native prints the inside cube only.
#[test]
fn print_apply_leaves_a_fully_outside_instance_out_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = with_outside_copy();
    let directory = tempfile::Builder::new()
        .prefix("print-volume-printable-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&path, &input).unwrap();
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
        .block_on(ares_core::slice_project_plate(
            input.as_slice(),
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
    let mut reader = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut text = Vec::new();
        file.read_to_end(&mut text).unwrap();
        let bytes = match file.name() {
            "3D/3dmodel.model" => String::from_utf8(text)
                .unwrap()
                .replacen(
                    "</build>",
                    "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 400 110 5\" printable=\"1\"/></build>",
                    1,
                )
                .into_bytes(),
            "Metadata/model_settings.config" => String::from_utf8(text)
                .unwrap()
                .replacen(
                    "  </plate>",
                    "    <model_instance>\n      <metadata key=\"object_id\" value=\"2\"/>\n      <metadata key=\"instance_id\" value=\"1\"/>\n      <metadata key=\"identify_id\" value=\"15\"/>\n    </model_instance>\n  </plate>",
                    1,
                )
                .into_bytes(),
            _ => text,
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
