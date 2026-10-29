use std::io::{Cursor, Read, Write};

pub(crate) fn add_volumes(
    project: &[u8],
    subtype: &str,
    transforms: &[(f64, f64)],
    options: &[(&str, &str)],
) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut annotation = Vec::new();
    archive
        .by_name("3D/Objects/overhang.stl_1.model")
        .unwrap()
        .read_to_end(&mut annotation)
        .unwrap();
    let annotation = String::from_utf8(annotation).unwrap();
    let mut components = String::new();
    let mut relationships = String::new();
    let mut parts = String::new();
    for (index, &(scale_x, shift_x)) in transforms.iter().enumerate() {
        let id = index + 3;
        components.push_str(&format!(
            "<component p:path=\"/3D/Objects/support-annotation-{id}.model\" objectid=\"{id}\" transform=\"{scale_x} 0 0 0 1 0 0 0 1 {shift_x} 0 7.5\"/>"
        ));
        relationships.push_str(&format!(
            "<Relationship Target=\"/3D/Objects/support-annotation-{id}.model\" Id=\"rel-annotation-{id}\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>"
        ));
        parts.push_str(&format!(
            "<part id=\"{id}\" subtype=\"{subtype}\"><metadata key=\"name\" value=\"overhang-annotation-{id}\"/><metadata key=\"matrix\" value=\"{scale_x} 0 0 {shift_x} 0 1 0 0 0 0 1 7.5 0 0 0 1\"/>"
        ));
        for &(key, value) in options {
            parts.push_str(&format!("<metadata key=\"{key}\" value=\"{value}\"/>"));
        }
        parts.push_str("</part>");
    }
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/3dmodel.model" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace("</components>", &format!("{components}</components>"))
                .into_bytes();
        } else if file.name() == "3D/_rels/3dmodel.model.rels" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "</Relationships>",
                    &format!("{relationships}</Relationships>"),
                )
                .into_bytes();
        } else if file.name() == "Metadata/model_settings.config" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace("</object>", &format!("{parts}</object>"))
                .into_bytes();
        } else if file.name() == "Metadata/project_settings.config"
            && let Some((_, material)) = options.iter().find(|(key, _)| *key == "filament_type")
        {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            set_filament_type(&mut settings, material);
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    for index in 0..transforms.len() {
        let id = index + 3;
        output
            .start_file(
                format!("3D/Objects/support-annotation-{id}.model"),
                zip::write::SimpleFileOptions::DEFAULT,
            )
            .unwrap();
        let model = annotation
            .replace("id=\"1\"", &format!("id=\"{id}\""))
            .replace("00010000-", &format!("000{id}0000-"));
        output.write_all(model.as_bytes()).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn set_filament_type(settings: &mut serde_json::Value, material: &str) {
    let Some(map) = settings.as_object_mut() else {
        return;
    };
    for (key, value) in map.iter_mut() {
        if (key == "different_settings_to_system" || key == "inherits_group")
            && value.as_array().is_some_and(|values| values.len() < 6)
        {
            while value.as_array().unwrap().len() < 6 {
                value
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::Value::String(String::new()));
            }
        }
        if (key.starts_with("filament_")
            || key.starts_with("printer_extruder")
            || key == "filament_map"
            || key == "filament_ids"
            || key == "filament_self_index")
            && value.as_array().is_some_and(|values| values.len() < 4)
        {
            let first = value.as_array().unwrap()[0].clone();
            while value.as_array().unwrap().len() < 4 {
                value.as_array_mut().unwrap().push(first.clone());
            }
        }
    }
    map.get_mut("filament_type")
        .unwrap()
        .as_array_mut()
        .unwrap()[1] = serde_json::Value::String(material.to_string());
}
