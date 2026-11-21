use std::fmt::Write;

use serde::Serialize;

use super::{BAMBU_VERSION, ObjectEntry, PLATE_GCODE};
use crate::ORCA_SLICER_COMPATIBILITY_VERSION as ORCA_VERSION;
use crate::{ProcessedGCode, SlicedArchive};

const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
const MODEL_OPEN: &str = concat!(
    "<model unit=\"millimeter\" xml:lang=\"en-US\" ",
    "xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" ",
    "xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" ",
    "xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n"
);
const VERSION_METADATA: &str = " <metadata name=\"BambuStudio:3mfVersion\">1</metadata>\n";
const IDENTITY: &str = "1 0 0 0 1 0 0 0 1";

pub(super) fn content_types() -> String {
    let mut out = format!(
        "{DECL}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n"
    );
    for (extension, media_type) in [
        (
            "rels",
            "application/vnd.openxmlformats-package.relationships+xml",
        ),
        (
            "model",
            "application/vnd.ms-package.3dmanufacturing-3dmodel+xml",
        ),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("json", "application/json"),
        ("gcode", "text/x.gcode"),
    ] {
        writeln!(
            out,
            " <Default Extension=\"{extension}\" ContentType=\"{media_type}\"/>"
        )
        .unwrap();
    }
    out.push_str("</Types>");
    out
}

pub(super) fn relationships<'a>(paths: impl IntoIterator<Item = &'a str>) -> String {
    relationship(paths, super::super::relationships::MODEL_RELATIONSHIP_TYPE)
}

pub(super) fn gcode_relationship() -> String {
    relationship(
        [PLATE_GCODE],
        "http://schemas.bambulab.com/package/2021/gcode",
    )
}

fn relationship<'a>(paths: impl IntoIterator<Item = &'a str>, media_type: &str) -> String {
    let mut out = format!(
        "{DECL}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n",
    );
    for (index, path) in paths.into_iter().enumerate() {
        let path = quick_xml::escape::escape(path);
        let id = index + 1;
        writeln!(
            out,
            " <Relationship Target=\"/{path}\" Id=\"rel-{id}\" Type=\"{media_type}\"/>"
        )
        .unwrap();
    }
    out.push_str("</Relationships>");
    out
}

pub(super) fn main_model(objects: &[ObjectEntry<'_>], loaded_orca_version: Option<&str>) -> String {
    let mut out = format!(
        "{DECL}{MODEL_OPEN} <metadata name=\"Application\">BambuStudio-{BAMBU_VERSION}</metadata>\n <metadata name=\"OrcaSlicer\">{ORCA_VERSION}</metadata>\n{VERSION_METADATA}"
    );
    for name in [
        "Copyright",
        "CreationDate",
        "Description",
        "Designer",
        "DesignerCover",
        "DesignerUserId",
        "License",
        "ModificationDate",
        "Origin",
        "Title",
    ] {
        if name == "Origin"
            && let Some(version) = loaded_orca_version
        {
            let version = quick_xml::escape::escape(version);
            writeln!(out, " <metadata name=\"OrcaSlicer\">{version}</metadata>").unwrap();
        }
        writeln!(out, " <metadata name=\"{name}\"></metadata>").unwrap();
    }
    out.push_str(" <resources>\n");
    for object in objects {
        let ObjectEntry {
            id,
            backup_id,
            path,
            ..
        } = object;
        let path = quick_xml::escape::escape(path);
        writeln!(out, "  <object id=\"{id}\" p:UUID=\"{backup_id:08x}-61cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <components>").unwrap();
        for (id, prefix, part) in object.parts() {
            let column_major = part.transform.to_column_major();
            let transform = numbers(
                column_major
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|column| &column[..3])
                    .copied(),
            );
            writeln!(out, "    <component p:path=\"/{path}\" objectid=\"{id}\" p:UUID=\"{prefix:08x}-b206-40ff-9872-83e8017abed1\" transform=\"{transform}\"/>").unwrap();
        }
        out.push_str("   </components>\n  </object>\n");
    }
    out.push_str(" </resources>\n <build p:UUID=\"2c7c17d8-22b5-4d84-8835-1976022ea369\">\n");
    for object in objects {
        let id = object.id;
        let [x, y, z] = object.object.placement.map(number);
        writeln!(out, "  <item objectid=\"{id}\" p:UUID=\"{id:08x}-b1ec-4553-aec9-835e5b724bb4\" transform=\"{IDENTITY} {x} {y} {z}\" printable=\"1\"/>").unwrap();
    }
    out.push_str(" </build>\n</model>\n");
    out
}

pub(super) fn mesh_model(object: &ObjectEntry<'_>) -> String {
    let mut out = format!("{DECL}{MODEL_OPEN}{VERSION_METADATA} <resources>\n");
    for (id, prefix, part) in object.parts() {
        writeln!(out, "  <object id=\"{id}\" p:UUID=\"{prefix:08x}-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <mesh>\n    <vertices>").unwrap();
        for [x, y, z] in &part.mesh.vertices {
            writeln!(out, "     <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
        }
        out.push_str("    </vertices>\n    <triangles>\n");
        for [a, b, c] in &part.mesh.triangles {
            writeln!(out, "     <triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>").unwrap();
        }
        out.push_str("    </triangles>\n   </mesh>\n  </object>\n");
    }
    out.push_str(" </resources>\n <build/>\n</model>\n");
    out
}

pub(super) fn model_settings(objects: &[ObjectEntry<'_>], sliced: bool) -> String {
    let mut out = format!("{DECL}<config>\n");
    for object in objects {
        let id = object.id;
        let name = quick_xml::escape::escape(&object.object.name);
        writeln!(out, "  <object id=\"{id}\" auto_drop=\"1\">\n    <metadata key=\"name\" value=\"{name}\"/>\n    <metadata key=\"extruder\" value=\"1\"/>").unwrap();
        for (id, _, part) in object.parts() {
            let name = quick_xml::escape::escape(&part.mesh.name);
            let filename = quick_xml::escape::escape(&part.source_file);
            let matrix = numbers(part.matrix.as_matrix().iter().flatten().copied());
            let [x, y, z] = part.mesh.source_offset.map(f64::from).map(number);
            let (source_object, source_volume) = (part.source_object, part.source_volume);
            writeln!(out, "    <part id=\"{id}\" subtype=\"normal_part\">\n      <metadata key=\"name\" value=\"{name}\"/>\n      <metadata key=\"matrix\" value=\"{matrix}\"/>").unwrap();
            if !filename.is_empty() {
                writeln!(out, "      <metadata key=\"source_file\" value=\"{filename}\"/>\n      <metadata key=\"source_object_id\" value=\"{source_object}\"/>\n      <metadata key=\"source_volume_id\" value=\"{source_volume}\"/>\n      <metadata key=\"source_offset_x\" value=\"{x}\"/>\n      <metadata key=\"source_offset_y\" value=\"{y}\"/>\n      <metadata key=\"source_offset_z\" value=\"{z}\"/>").unwrap();
            }
            out.push_str("      <mesh_stat edges_fixed=\"0\" degenerate_facets=\"0\" facets_removed=\"0\" facets_reversed=\"0\" backwards_edges=\"0\"/>\n    </part>\n");
        }
        out.push_str("  </object>\n");
    }
    let gcode_file = if sliced { PLATE_GCODE } else { "" };
    writeln!(out, "  <plate>\n    <metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"{gcode_file}\"/>").unwrap();
    for object in objects {
        if let Some(identify_id) = object.identify_id {
            let id = object.id;
            writeln!(out, "    <model_instance>\n      <metadata key=\"object_id\" value=\"{id}\"/>\n      <metadata key=\"instance_id\" value=\"0\"/>\n      <metadata key=\"identify_id\" value=\"{identify_id}\"/>\n    </model_instance>").unwrap();
        }
    }
    out.push_str("  </plate>\n  <assemble>\n  </assemble>\n</config>\n");
    out
}

pub(super) fn plate_json(archive: &SlicedArchive, first_print_object_id: u32) -> Vec<u8> {
    #[derive(Serialize)]
    struct Object<'a> {
        area: f64,
        bbox: [f64; 4],
        id: u32,
        layer_height: f64,
        name: &'a str,
    }
    #[derive(Serialize)]
    struct Plate<'a> {
        bbox_all: [f64; 4],
        bbox_objects: Vec<Object<'a>>,
        bed_type: &'static str,
        filament_colors: [String; 1],
        filament_ids: [u32; 1],
        first_extruder: u32,
        first_layer_time: f32,
        is_seq_print: bool,
        nozzle_diameter: f64,
        version: u32,
    }
    let bbox_all = archive
        .objects
        .iter()
        .map(|object| object.bounds)
        .reduce(|a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        })
        .expect("sliced plate has objects");
    let bbox_objects = archive
        .objects
        .iter()
        .enumerate()
        .map(|(index, object)| Object {
            area: f64::from(object.area),
            bbox: object.bounds,
            id: first_print_object_id + index as u32,
            layer_height: f64::from(object.layer_height),
            name: &object.name,
        })
        .collect();
    serde_json::to_vec(&Plate {
        bbox_all,
        bbox_objects,
        bed_type: "cool_plate",
        filament_colors: [archive.filament_color.clone()],
        filament_ids: [0],
        first_extruder: 0,
        first_layer_time: 0.0,
        is_seq_print: false,
        nozzle_diameter: f64::from(archive.nozzle_diameter),
        version: 2,
    })
    .expect("plate metadata is serializable")
}

pub(super) fn slice_info(archive: &SlicedArchive, processed: &ProcessedGCode) -> String {
    // PartPlate.cpp:8470–8481: integer seconds and std::to_string(float).
    let prediction = processed.result.print_statistics.normal_time as u32;
    let first_layer_time = format!("{:.6}", processed.result.initial_layer_time);
    let weight = processed.print_statistics.total_weight;
    let used_m = processed.print_statistics.total_used_filament / 1000.0;
    let used_g = processed.print_statistics.total_weight;
    let last_layer = archive.layer_count.saturating_sub(1);
    let mut objects = archive.objects.iter().collect::<Vec<_>>();
    objects.sort_by_key(|object| object.model_index);
    let mut object_entries = String::new();
    for object in objects {
        let name = quick_xml::escape::escape(&object.name);
        let identify_id = object.identify_id;
        writeln!(
            object_entries,
            "    <object identify_id=\"{identify_id}\" name=\"{name}\" skipped=\"false\" />"
        )
        .unwrap();
    }
    format!(
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n  <plate>\n    <metadata key=\"index\" value=\"1\"/>\n    <metadata key=\"extruder_type\" value=\"0\"/>\n    <metadata key=\"nozzle_volume_type\" value=\"0\"/>\n    <metadata key=\"printer_model_id\" value=\"\"/>\n    <metadata key=\"nozzle_diameters\" value=\"{:.1}\"/>\n    <metadata key=\"timelapse_type\" value=\"0\"/>\n    <metadata key=\"prediction\" value=\"{prediction}\"/>\n    <metadata key=\"weight\" value=\"{weight:.2}\"/>\n    <metadata key=\"first_layer_time\" value=\"{first_layer_time}\"/>\n    <metadata key=\"outside\" value=\"false\"/>\n    <metadata key=\"support_used\" value=\"false\"/>\n    <metadata key=\"label_object_enabled\" value=\"true\"/>\n    <metadata key=\"enable_filament_dynamic_map\" value=\"false\"/>\n    <metadata key=\"has_filament_switcher\" value=\"false\"/>\n    <metadata key=\"filament_maps\" value=\"1\"/>\n    <metadata key=\"limit_filament_maps\" value=\"0\"/>\n{object_entries}    <filament id=\"1\" tray_info_idx=\"{}\" type=\"{}\" color=\"{}\" used_m=\"{used_m:.2}\" used_g=\"{used_g:.2}\" group_id=\"0\" nozzle_diameter=\"{:.2}\" volume_type=\"Standard\" used_for_object=\"true\" used_for_support=\"false\"/>\n    <nozzle id=\"0\" extruder_id=\"1\" nozzle_diameter=\"{:.1}\" volume_type=\"Standard\"/>\n    <layer_filament_lists>\n      <layer_filament_list filament_list=\"0\" layer_ranges=\"0 {last_layer}\" />\n    </layer_filament_lists>\n  </plate>\n</config>\n",
        archive.nozzle_diameter,
        archive.filament_id,
        archive.filament_type,
        archive.filament_color,
        archive.nozzle_diameter,
        archive.nozzle_diameter,
    )
}

pub(super) fn filament_sequence() -> Vec<u8> {
    b"{\"plate_1\":{\"nozzle_sequence\":[0],\"optimal_assignment\":[0],\"sequence\":[1]}}".to_vec()
}

pub(super) fn slice_info_header() -> String {
    format!(
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n</config>\n"
    )
}

fn numbers(values: impl Iterator<Item = f64>) -> String {
    values.map(number).collect::<Vec<_>>().join(" ")
}

fn number(value: f64) -> String {
    crate::options::format_number(value)
}
