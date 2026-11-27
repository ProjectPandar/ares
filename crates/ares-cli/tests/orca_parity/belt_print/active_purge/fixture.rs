use serde_json::{Map, Value, json};
use std::io::{Cursor, Write as _};
use zip::{ZipWriter, write::SimpleFileOptions};

const PART: &str = "3D/Objects/cube10.stl_1.model";
const PRISM: &str = "3D/Objects/purge-prism.model";

pub(super) fn project(matrix: [&str; 2], use_matrix: bool, solid_tool: &str) -> Vec<u8> {
    let prism_mesh = std::cell::OnceCell::new();
    let input = super::super::project_with(true, false, &[], false, |member, bytes| {
        match member {
            "Metadata/project_settings.config" => settings(&bytes, matrix, use_matrix),
            "Metadata/model_settings.config" => {
                model_settings(std::str::from_utf8(&bytes).unwrap(), solid_tool).into_bytes()
            }
            "3D/3dmodel.model" => plate(std::str::from_utf8(&bytes).unwrap()).into_bytes(),
            "3D/_rels/3dmodel.model.rels" => {
                let xml = std::str::from_utf8(&bytes).unwrap();
                let link = fragment(xml, "<Relationship Target=", "/>");
                append_after(
                    xml,
                    link,
                    &link.replace(PART, PRISM).replace("rel-1", "rel-2"),
                )
                .into_bytes()
            }
            PART => {
                let mesh = std::str::from_utf8(&bytes).unwrap();
                // Scale the 2 mm source part to a 6 mm prism.
                assert_eq!(mesh.matches("z=\"2\"").count(), 4);
                prism_mesh
                    .set(
                        mesh.replace("z=\"2\"", "z=\"6\"")
                            .replace("UUID=\"0", "UUID=\"3"),
                    )
                    .unwrap();
                bytes
            }
            _ => bytes,
        }
    });
    let mut zip = ZipWriter::new_append(Cursor::new(input)).unwrap();
    zip.start_file(PRISM, SimpleFileOptions::default()).unwrap();
    zip.write_all(prism_mesh.into_inner().unwrap().as_bytes())
        .unwrap();
    zip.finish().unwrap().into_inner()
}

fn settings(bytes: &[u8], matrix: [&str; 2], use_matrix: bool) -> Vec<u8> {
    let mut config: Map<String, Value> = serde_json::from_slice(bytes).unwrap();
    for (key, value) in &mut config {
        if key.starts_with("filament_")
            && let Some([first]) = value.as_array().map(Vec::as_slice)
        {
            *value = json!([first, first]);
        }
    }
    config.extend(
        [
            ("belt_slice_rotation_angle", json!("45")),
            ("brim_type", json!("no_brim")),
            ("enable_belt_purge_tower", json!("1")),
            ("enable_prime_tower", json!("1")),
            ("use_relative_e_distances", json!("1")),
            ("filament_colour", json!(["#ff0000", "#0000ff"])),
            ("flush_multiplier", json!(["0.5"])),
            (
                "flush_volumes_matrix",
                json!(["0", matrix[0], matrix[1], "0"]),
            ),
            ("flush_volumes_vector", json!(vec!["140"; 4])),
            ("inherits_group", json!(vec![""; 4])),
            (
                "purge_in_prime_tower",
                json!(if use_matrix { "1" } else { "0" }),
            ),
            ("prime_volume", json!(if use_matrix { "45" } else { "3" })),
            (
                "change_filament_gcode",
                json!("; swap [previous_extruder] -> [next_extruder]"),
            ),
        ]
        .map(|(key, value)| (key.to_owned(), value)),
    );
    serde_json::to_vec(&config).unwrap()
}

fn plate(xml: &str) -> String {
    let model = fragment(xml, "<object id=\"2\"", "</object>");
    let prism = model
        .replace("id=\"2\"", "id=\"3\"")
        .replace("UUID=\"0", "UUID=\"3")
        .replace(PART, PRISM);
    let item = fragment(xml, "<item objectid=\"2\"", "/>");
    let prism_item = item
        .replace("objectid=\"2\"", "objectid=\"3\"")
        .replace("UUID=\"0", "UUID=\"3")
        .replace("110 110 0\"", "145 110.07 0\"");
    append_after(&append_after(xml, model, &prism), item, &prism_item)
}

fn model_settings(xml: &str, solid_tool: &str) -> String {
    let model = fragment(xml, "<object id=\"2\"", "</object>");
    let prism = model
        .replace("id=\"2\"", "id=\"3\"")
        .replace(
            "key=\"name\" value=\"cube10.stl\"",
            "key=\"name\" value=\"purge-prism\"",
        )
        .replacen(
            "<part ",
            "<metadata key=\"belt_purge_tower_object\" value=\"1\"/>\n    \
             <metadata key=\"flush_into_objects\" value=\"1\"/>\n    <part ",
            1,
        );
    let models = format!(
        "{}\n  {prism}",
        model.replacen(
            "<part ",
            &format!(
                "<metadata key=\"internal_solid_filament_id\" value=\"{solid_tool}\"/>\n    <part "
            ),
            1,
        )
    );
    let instance = fragment(xml, "<model_instance>", "</model_instance>");
    let prism_instance = instance
        .replace(
            "key=\"object_id\" value=\"2\"",
            "key=\"object_id\" value=\"3\"",
        )
        .replace(
            "key=\"identify_id\" value=\"14\"",
            "key=\"identify_id\" value=\"15\"",
        );
    append_after(&xml.replacen(model, &models, 1), instance, &prism_instance)
}

fn fragment<'a>(xml: &'a str, open: &str, close: &str) -> &'a str {
    let start = xml.find(open).expect("fixture contains source element");
    let tail = &xml[start..];
    &tail[..tail.find(close).unwrap() + close.len()]
}

fn append_after(xml: &str, old: &str, new: &str) -> String {
    xml.replacen(old, &format!("{old}\n  {new}"), 1)
}
