use std::{
    fmt::Write as _,
    io::{Cursor, Read, Write as _},
};

use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const CUBE_MESH: &str = "3D/Objects/cube10.stl_1.model";
const PART: Volume = Volume {
    subtype: "normal_part",
    scale: [1.0, 1.0, 1.0],
    offset: [0.0, 6.0, 0.0],
    options: &[],
};
const MODIFIER: Volume = Volume {
    subtype: "modifier_part",
    scale: [0.6, 1.0, 1.0],
    offset: [4.0, 0.0, 0.0],
    options: &[("wall_loops", "4"), ("sparse_infill_density", "40%")],
};

#[test]
fn slices_overlapping_model_parts_to_the_end_of_the_belt_span() {
    compare("overlapping-parts", &[PART]);
}

#[test]
fn applies_region_settings_inside_the_belt_modifier() {
    compare("wall-modifier", &[MODIFIER]);
}

#[test]
fn applies_one_belt_modifier_to_both_model_parts() {
    compare(
        "modifier-spans-parts",
        &[
            Volume {
                offset: [8.0, 6.0, 0.0],
                ..PART
            },
            Volume {
                scale: [1.0, 1.6, 1.0],
                ..MODIFIER
            },
        ],
    );
}

struct Volume {
    subtype: &'static str,
    scale: [f64; 3],
    offset: [f64; 3],
    options: &'static [(&'static str, &'static str)],
}

fn compare(name: &str, volumes: &[Volume]) {
    let project = super::project(true, false, &[], false);
    super::compare_project(name, with_volumes(&project, volumes));
}

fn with_volumes(project: &[u8], volumes: &[Volume]) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    let mut mesh = String::new();
    archive
        .by_name(CUBE_MESH)
        .unwrap()
        .read_to_string(&mut mesh)
        .unwrap();

    let mut components = String::new();
    let mut relationships = String::new();
    let mut parts = String::new();
    for (index, volume) in volumes.iter().enumerate() {
        let id = index + 3;
        let path = format!("3D/Objects/belt-volume-{id}.model");
        let Volume {
            subtype,
            scale: [sx, sy, sz],
            offset: [x, y, z],
            options,
        } = volume;
        write!(components, r#"<component p:path="/{path}" objectid="{id}" transform="{sx} 0 0 0 {sy} 0 0 0 {sz} {x} {y} {z}"/>"#).unwrap();
        write!(relationships, r#"<Relationship Target="/{path}" Id="rel-{id}" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>"#).unwrap();
        write!(parts, r#"<part id="{id}" subtype="{subtype}"><metadata key="name" value="volume-{id}"/><metadata key="matrix" value="{sx} 0 0 {x} 0 {sy} 0 {y} 0 0 {sz} {z} 0 0 0 1"/>"#).unwrap();
        for (key, value) in *options {
            write!(parts, r#"<metadata key="{key}" value="{value}"/>"#).unwrap();
        }
        parts.push_str("</part>");
        output
            .start_file(path, SimpleFileOptions::default())
            .unwrap();
        let model = mesh
            .replace("id=\"1\"", &format!("id=\"{id}\""))
            .replace("00010000-", &format!("{id:04}0000-"));
        output.write_all(model.as_bytes()).unwrap();
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let xml = match file.name() {
            "3D/3dmodel.model" => Some(("</components>", &components)),
            "3D/_rels/3dmodel.model.rels" => Some(("</Relationships>", &relationships)),
            "Metadata/model_settings.config" => Some(("</object>", &parts)),
            _ => None,
        };
        if let Some((end, insertion)) = xml {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(end, &format!("{insertion}{end}"))
                .into_bytes();
        }
        output
            .start_file(file.name(), SimpleFileOptions::default())
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}
