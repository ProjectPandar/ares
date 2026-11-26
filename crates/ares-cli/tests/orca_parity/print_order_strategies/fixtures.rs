//! Procedural 3MF fixture builders for `print_order = snake` and `best_of` parity cases.

use std::io::{Cursor, Read, Write};

use serde_json::{Value, json};

const BASE_SINGLE_3MF: &[u8] = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");
const BASE_TWO_OBJ_3MF: &[u8] =
    include_bytes!("../../../../../tests/parity/print-order-as-obj-list/class-default.3mf");

pub(super) struct BoxSpec {
    pub(super) cx: f64,
    pub(super) cy: f64,
    pub(super) z0: f64,
    pub(super) z1: f64,
    pub(super) sx: f64,
    pub(super) sy: f64,
}

impl BoxSpec {
    pub(super) const fn cube4(cx: f64, cy: f64) -> Self {
        Self {
            cx,
            cy,
            z0: -0.3,
            z1: 0.3,
            sx: 4.0,
            sy: 4.0,
        }
    }
}

pub(super) fn build_single_object_3mf(
    boxes: &[BoxSpec],
    instances: &[[f64; 3]],
    settings_patch: &[(&str, Value)],
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(BASE_SINGLE_3MF)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(member.compression())
            .last_modified_time(zip::DateTime::default());
        if member.is_dir() {
            output.add_directory(member.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match member.name() {
            "3D/Objects/cube10.stl_1.model" => {
                bytes = render_model_xml(boxes, 1).into_bytes();
            }
            "3D/3dmodel.model" => {
                let text = String::from_utf8(bytes).unwrap();
                bytes = replace_build_items(&text, &[(2, instances)]).into_bytes();
            }
            "Metadata/model_settings.config" => {
                let text = String::from_utf8(bytes).unwrap();
                bytes = replace_model_instances(&text, &[(2, instances.len())]).into_bytes();
            }
            "Metadata/project_settings.config" => {
                bytes = patch_project_settings(&bytes, settings_patch);
            }
            _ => {}
        }
        output.start_file(member.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

pub(super) struct ObjectSpec<'a> {
    pub(super) boxes: &'a [BoxSpec],
    pub(super) instances: &'a [[f64; 3]],
    pub(super) extruder: u32,
}

pub(super) fn build_two_object_3mf(
    first: ObjectSpec<'_>,
    second: ObjectSpec<'_>,
    settings_patch: &[(&str, Value)],
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(BASE_TWO_OBJ_3MF)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(member.compression())
            .last_modified_time(zip::DateTime::default());
        if member.is_dir() {
            output.add_directory(member.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match member.name() {
            "3D/Objects/cube10.stl_1.model" => {
                bytes = render_model_xml(first.boxes, 1).into_bytes();
            }
            "3D/Objects/cube10.stl_2.model" => {
                bytes = render_model_xml(second.boxes, 3).into_bytes();
            }
            "3D/3dmodel.model" => {
                let text = String::from_utf8(bytes).unwrap();
                bytes = replace_build_items(&text, &[(2, first.instances), (4, second.instances)])
                    .into_bytes();
            }
            "Metadata/model_settings.config" => {
                let mut text = String::from_utf8(bytes).unwrap();
                text = set_object_extruder(&text, 2, first.extruder);
                text = set_object_extruder(&text, 4, second.extruder);
                bytes = replace_model_instances(
                    &text,
                    &[(2, first.instances.len()), (4, second.instances.len())],
                )
                .into_bytes();
            }
            "Metadata/project_settings.config" => {
                bytes = patch_project_settings(&bytes, settings_patch);
            }
            _ => {}
        }
        output.start_file(member.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn patch_project_settings(bytes: &[u8], settings_patch: &[(&str, Value)]) -> Vec<u8> {
    let mut settings: Value = serde_json::from_slice(bytes).unwrap();
    let map = settings.as_object_mut().unwrap();
    map.insert("layer_height".into(), json!("0.2"));
    map.insert("initial_layer_print_height".into(), json!("0.2"));
    map.insert("top_shell_layers".into(), json!("1"));
    map.insert("bottom_shell_layers".into(), json!("1"));
    map.insert("wall_loops".into(), json!("1"));
    map.insert("sparse_infill_density".into(), json!("0%"));
    map.insert("skirt_loops".into(), json!("0"));
    map.insert("brim_type".into(), json!("no_brim"));
    map.insert("enable_prime_tower".into(), json!("0"));
    map.insert("gcode_label_objects".into(), json!("1"));
    map.insert("wipe_tower_x".into(), json!(["15"]));
    map.insert("wipe_tower_y".into(), json!(["220"]));
    for (key, value) in settings_patch {
        map.insert((*key).into(), value.clone());
    }
    serde_json::to_vec_pretty(&settings).unwrap()
}

fn replace_build_items(xml: &str, groups: &[(u32, &[[f64; 3]])]) -> String {
    let start = xml.find("<build ").expect("build open tag");
    let end = xml.find("</build>").expect("build close tag") + "</build>".len();
    let mut build = String::from("<build p:UUID=\"2c9e5b3a-7d1c-4a62-9f4e-2a7c8b9e1d2f\">\n");
    let mut uid = 0usize;
    for &(object_id, instances) in groups {
        for &[x, y, z] in instances {
            build.push_str(&format!(
                "  <item objectid=\"{object_id}\" p:UUID=\"00030000-b206-40ff-9872-83e8017ab{uid:03x}\" \
                 transform=\"1 0 0 0 1 0 0 0 1 {x} {y} {z}\" printable=\"1\"/>\n"
            ));
            uid += 1;
        }
    }
    build.push_str(" </build>");
    format!("{}{build}{}", &xml[..start], &xml[end..])
}

fn replace_model_instances(xml: &str, groups: &[(u32, usize)]) -> String {
    let start = xml.find("<model_instance>").expect("first model_instance");
    let end =
        xml.rfind("</model_instance>").expect("last model_instance") + "</model_instance>".len();
    let mut rendered = String::new();
    let mut identify_id = 100usize;
    for &(object_id, count) in groups {
        for instance_id in 0..count {
            if !rendered.is_empty() {
                rendered.push('\n');
            }
            rendered.push_str(&format!(
                "    <model_instance>\n\
                 \x20     <metadata key=\"object_id\" value=\"{object_id}\"/>\n\
                 \x20     <metadata key=\"instance_id\" value=\"{instance_id}\"/>\n\
                 \x20     <metadata key=\"identify_id\" value=\"{identify_id}\"/>\n\
                 \x20   </model_instance>"
            ));
            identify_id += 1;
        }
    }
    format!("{}{rendered}{}", &xml[..start], &xml[end..])
}

fn set_object_extruder(xml: &str, object_id: u32, extruder: u32) -> String {
    let header = format!("<object id=\"{object_id}\">");
    let obj_start = xml.find(&header).expect("object block");
    let key = "<metadata key=\"extruder\" value=\"";
    let key_rel = xml[obj_start..].find(key).expect("extruder metadata");
    let val_start = obj_start + key_rel + key.len();
    let val_end = val_start + xml[val_start..].find('"').expect("closing quote");
    format!("{}{extruder}{}", &xml[..val_start], &xml[val_end..])
}

fn render_model_xml(boxes: &[BoxSpec], object_id: u32) -> String {
    let mut vertices = Vec::with_capacity(boxes.len() * 8);
    let mut triangles = Vec::with_capacity(boxes.len() * 12);
    for b in boxes {
        push_box(
            &mut vertices,
            &mut triangles,
            [b.cx - b.sx / 2.0, b.cy - b.sy / 2.0, b.z0],
            [b.cx + b.sx / 2.0, b.cy + b.sy / 2.0, b.z1],
        );
    }
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <model unit=\"millimeter\" xml:lang=\"en-US\" \
         xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" \
         xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" \
         xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" \
         requiredextensions=\"p\">\n\
          <metadata name=\"BambuStudio:3mfVersion\">1</metadata>\n\
          <resources>\n\
           <object id=\"{object_id}\" p:UUID=\"000{object_id}0000-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n\
            <mesh>\n\
             <vertices>\n"
    );
    for [x, y, z] in &vertices {
        xml.push_str(&format!("      <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>\n"));
    }
    xml.push_str("     </vertices>\n     <triangles>\n");
    for [a, b, c] in &triangles {
        xml.push_str(&format!(
            "      <triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>\n"
        ));
    }
    xml.push_str(
        "     </triangles>\n    </mesh>\n   </object>\n  </resources>\n  <build/>\n</model>\n",
    );
    xml
}

fn push_box(
    vertices: &mut Vec<[f64; 3]>,
    triangles: &mut Vec<[usize; 3]>,
    min: [f64; 3],
    max: [f64; 3],
) {
    let base = vertices.len();
    vertices.extend_from_slice(&[
        [max[0], min[1], min[2]],
        [min[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [min[0], max[1], max[2]],
    ]);
    let faces = [
        [0, 1, 2],
        [2, 1, 3],
        [4, 5, 6],
        [7, 5, 4],
        [5, 1, 6],
        [6, 1, 0],
        [2, 3, 4],
        [4, 3, 7],
        [3, 1, 7],
        [7, 1, 5],
        [6, 0, 4],
        [4, 0, 2],
    ];
    for [a, b, c] in faces {
        triangles.push([base + a, base + b, base + c]);
    }
}
