use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use super::mapped_nozzle::{mapped_project, outer_feedrates};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Write},
};

fn object_override(source: &[u8], token: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).unwrap();
            let xml = std::str::from_utf8(&bytes).unwrap();
            let marker = "<object id=\"2\">";
            assert_eq!(xml.matches(marker).count(), 1);
            let updated = xml.replace(
                marker,
                &format!("{marker}\n    <metadata key=\"outer_wall_speed\" value=\"{token}\"/>"),
            );
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(updated.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn native_region_overlays_remap_variants_and_inherit_nil() {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    let mut failures = Vec::new();
    for (case, token, physical, expected_feed) in [
        ("multi", "87,99,113,127", 2, "6780"),
        ("singleton", "83", 2, "4980"),
        ("nil", "nil", 2, "1980"),
        ("source-equal", "17.00009,33", 1, "1020"),
        ("source-different", "17.00011,33", 1, "1020.007"),
    ] {
        let mapped = mapped_project(&source, physical);
        let project = object_override(&mapped, token);
        let (reference, _) = native_stream_case::qualify(
            (
                "nullable-region-overlay",
                &[
                    "PrintObject.cpp:4363-4374 apply_to_print_region_config",
                    "PrintConfig.cpp:11421-11429 set_variant_override",
                    "Config.hpp:612-629 set_to_index nil inherits existing target",
                    "Config.hpp:985-995; libslic3r.h:320-323 option equality precedes overlay",
                ],
            ),
            case,
            &project,
            &sha(&project),
            false,
        );
        let native = outer_feedrates(&reference.native);
        assert_eq!(native, ["3000".to_owned(), expected_feed.to_owned()].into());
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ares_core::slice_project_plate_validated(
                &project,
                GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ConfigOverrides::default(),
                SlicerContext::new(0),
            ));
        let api = result.as_ref().ok().map(|bytes| {
            fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
            outer_feedrates(std::str::from_utf8(bytes).unwrap())
        });
        let matches = api.as_ref() == Some(&native);
        fs::write(
            reference.root.join("region-overlay-comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "case": case, "authored_token": token, "physical_extruder": physical,
                "input_sha256": sha(&project), "native": native, "api": api,
                "channel_matches": matches,
                "api_error": result.as_ref().err().map(|error| format!("{error:?}")),
                "complete_gcode_parity_claimed": false,
            }))
            .unwrap(),
        )
        .unwrap();
        if !matches {
            failures.push(reference.root);
        }
    }
    assert!(
        failures.is_empty(),
        "region overlay public output: {failures:?}"
    );
}
