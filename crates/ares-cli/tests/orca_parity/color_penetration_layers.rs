//! MultiMaterialSegmentation.cpp:1241-1582: painted horizontal surface depth.

use crate::runner::{native_api_cli::sha, native_stream_case};
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

const UPSTREAM: &[&str] = &[
    "MultiMaterialSegmentation.cpp:1241-1582",
    "MultiMaterialSegmentation.cpp:1999-2088,2449-2455",
];
const BASE: &[u8] = include_bytes!("../../../../tests/parity/painted-side-cut/split-d0.3mf");

fn project(painted: bool) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let side_paint = regex::Regex::new(r#" paint_color="[^"]*""#).unwrap();
    let component = regex::Regex::new(r#"(?m)^.*<component[^>]*overlay-box[^>]*/>\n"#).unwrap();
    let part = regex::Regex::new(r#"(?s)    <part id="2".*?</part>\n"#).unwrap();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings = serde_json::from_slice::<Value>(&bytes).unwrap();
            for (key, value) in [
                ("top_color_penetration_layers", "4"),
                ("bottom_color_penetration_layers", "0"),
                ("top_shell_layers", "2"),
                ("bottom_shell_layers", "2"),
                ("mmu_segmented_region_max_width", "0"),
                ("mmu_segmented_region_interlocking_depth", "0"),
            ] {
                settings[key] = json!(value);
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else if entry.name() == "3D/3dmodel.model" {
            let xml = String::from_utf8(bytes).unwrap();
            bytes = component.replace_all(&xml, "").as_bytes().to_vec();
        } else if entry.name() == "Metadata/model_settings.config" {
            let xml = String::from_utf8(bytes).unwrap();
            bytes = part.replace_all(&xml, "").as_bytes().to_vec();
        } else if entry.name() == "3D/Objects/base-box.stl_1.model" {
            let xml = String::from_utf8(bytes).unwrap();
            let mut xml = side_paint.replace_all(&xml, "").into_owned();
            // Both upward-facing triangles at z=10, state 2 (unsplit code 8).
            for triangle in [
                r#"<triangle v1="4" v2="5" v3="6"/>"#,
                r#"<triangle v1="7" v2="5" v3="4"/>"#,
            ]
            .into_iter()
            .filter(|_| painted)
            {
                assert!(xml.contains(triangle));
                xml = xml.replace(triangle, &triangle.replace("/>", " paint_color=\"8\"/>"));
            }
            bytes = xml.into_bytes();
        }
        writer
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn check(painted: bool, label: &str) {
    let project = project(painted);
    native_stream_case::check(
        ("color-penetration-layers", UPSTREAM),
        label,
        &project,
        &sha(&project),
        (painted, |reference, replay| {
            for native in [reference, replay] {
                let activity = crate::paint_color_activity::activity(native.native.as_bytes());
                let layers = activity["positive_model_XYE_by_layer"].as_array().unwrap();
                let colored = layers
                    .iter()
                    .filter(|layer| layer["1"].as_u64().unwrap_or(0) > 0)
                    .count();
                if painted {
                    assert!(colored > 2, "penetration must exceed the two shell layers");
                } else {
                    assert_eq!(
                        colored, 0,
                        "removing the patch removes colored model extrusion"
                    );
                }
            }
        }),
    );
}

#[test]
fn color_penetration_layers_top_patch_matches_native() {
    check(true, "top-patch-four-layers");
}

#[test]
fn color_penetration_layers_without_patch_matches_native() {
    check(false, "without-top-patch");
}
