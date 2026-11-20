use std::fmt::Write;

use serde::Serialize;

use super::{BAMBU_VERSION, PLATE_GCODE, geometry::Mesh};
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

pub(super) fn relationships(path: &str) -> String {
    relationship(path, super::super::relationships::MODEL_RELATIONSHIP_TYPE)
}

pub(super) fn gcode_relationship() -> String {
    relationship(
        PLATE_GCODE,
        "http://schemas.bambulab.com/package/2021/gcode",
    )
}

fn relationship(path: &str, media_type: &str) -> String {
    let path = quick_xml::escape::escape(path);
    format!(
        "{DECL}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n <Relationship Target=\"/{path}\" Id=\"rel-1\" Type=\"{media_type}\"/>\n</Relationships>",
    )
}

pub(super) fn main_model(mesh: &Mesh, part: &str, loaded_orca_version: Option<&str>) -> String {
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
    let part = quick_xml::escape::escape(part);
    let [x, y, z] = mesh.placement.map(number);
    write!(out, " <resources>\n  <object id=\"2\" p:UUID=\"00000001-61cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <components>\n    <component p:path=\"/{part}\" objectid=\"1\" p:UUID=\"00010000-b206-40ff-9872-83e8017abed1\" transform=\"{IDENTITY} 0 0 0\"/>\n   </components>\n  </object>\n </resources>\n <build p:UUID=\"2c7c17d8-22b5-4d84-8835-1976022ea369\">\n  <item objectid=\"2\" p:UUID=\"00000002-b1ec-4553-aec9-835e5b724bb4\" transform=\"{IDENTITY} {x} {y} {z}\" printable=\"1\"/>\n </build>\n</model>").unwrap();
    out.push('\n');
    out
}

pub(super) fn mesh_model(mesh: &Mesh) -> String {
    let mut out = format!(
        "{DECL}{MODEL_OPEN}{VERSION_METADATA} <resources>\n  <object id=\"1\" p:UUID=\"00010000-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <mesh>\n    <vertices>\n"
    );
    for [x, y, z] in &mesh.vertices {
        writeln!(out, "     <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
    }
    out.push_str("    </vertices>\n    <triangles>\n");
    for [a, b, c] in &mesh.triangles {
        writeln!(out, "     <triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>").unwrap();
    }
    out.push_str("    </triangles>\n   </mesh>\n  </object>\n </resources>\n <build/>\n</model>\n");
    out
}

pub(super) fn model_settings(
    mesh: &Mesh,
    filename: &str,
    sliced: bool,
    identify_id: Option<u32>,
) -> String {
    let name = quick_xml::escape::escape(&mesh.name);
    let filename = quick_xml::escape::escape(filename);
    let [x, y, z] = mesh.source_offset.map(f64::from).map(number);
    let gcode_file = if sliced { PLATE_GCODE } else { "" };
    let model_instance = identify_id.map_or_else(String::new, |identify_id| {
        format!(
            "    <model_instance>\n      <metadata key=\"object_id\" value=\"2\"/>\n      <metadata key=\"instance_id\" value=\"0\"/>\n      <metadata key=\"identify_id\" value=\"{identify_id}\"/>\n    </model_instance>\n"
        )
    });
    format!(
        "{DECL}<config>\n  <object id=\"2\" auto_drop=\"1\">\n    <metadata key=\"name\" value=\"{name}\"/>\n    <metadata key=\"extruder\" value=\"1\"/>\n    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"name\" value=\"{name}\"/>\n      <metadata key=\"matrix\" value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"/>\n      <metadata key=\"source_file\" value=\"{filename}\"/>\n      <metadata key=\"source_object_id\" value=\"0\"/>\n      <metadata key=\"source_volume_id\" value=\"0\"/>\n      <metadata key=\"source_offset_x\" value=\"{x}\"/>\n      <metadata key=\"source_offset_y\" value=\"{y}\"/>\n      <metadata key=\"source_offset_z\" value=\"{z}\"/>\n      <mesh_stat edges_fixed=\"0\" degenerate_facets=\"0\" facets_removed=\"0\" facets_reversed=\"0\" backwards_edges=\"0\"/>\n    </part>\n  </object>\n  <plate>\n    <metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"{gcode_file}\"/>\n{model_instance}  </plate>\n  <assemble>\n  </assemble>\n</config>\n"
    )
}

pub(super) fn plate_json(archive: &SlicedArchive, print_object_id: u32) -> Vec<u8> {
    #[derive(Serialize)]
    struct Object {
        area: f32,
        bbox: [f64; 4],
        id: u32,
        layer_height: f64,
        name: String,
    }
    #[derive(Serialize)]
    struct Plate {
        bbox_all: [f64; 4],
        bbox_objects: [Object; 1],
        bed_type: &'static str,
        filament_colors: [String; 1],
        filament_ids: [u32; 1],
        first_extruder: u32,
        first_layer_time: f32,
        is_seq_print: bool,
        nozzle_diameter: f64,
        version: u32,
    }
    serde_json::to_vec(&Plate {
        bbox_all: archive.bounds,
        bbox_objects: [Object {
            area: archive.area,
            bbox: archive.bounds,
            id: print_object_id,
            layer_height: f64::from(archive.layer_height),
            name: archive.object_name.clone(),
        }],
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
    let prediction = processed.result.print_statistics.normal_time.round() as u32;
    let weight = processed.print_statistics.total_weight;
    let used_m = processed.print_statistics.total_used_filament / 1000.0;
    let used_g = processed.print_statistics.total_weight;
    let last_layer = archive.layer_count.saturating_sub(1);
    format!(
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n  <plate>\n    <metadata key=\"index\" value=\"1\"/>\n    <metadata key=\"extruder_type\" value=\"0\"/>\n    <metadata key=\"nozzle_volume_type\" value=\"0\"/>\n    <metadata key=\"printer_model_id\" value=\"\"/>\n    <metadata key=\"nozzle_diameters\" value=\"{:.1}\"/>\n    <metadata key=\"timelapse_type\" value=\"0\"/>\n    <metadata key=\"prediction\" value=\"{prediction}\"/>\n    <metadata key=\"weight\" value=\"{weight:.2}\"/>\n    <metadata key=\"first_layer_time\" value=\"{}\"/>\n    <metadata key=\"outside\" value=\"false\"/>\n    <metadata key=\"support_used\" value=\"false\"/>\n    <metadata key=\"label_object_enabled\" value=\"true\"/>\n    <metadata key=\"enable_filament_dynamic_map\" value=\"false\"/>\n    <metadata key=\"has_filament_switcher\" value=\"false\"/>\n    <metadata key=\"filament_maps\" value=\"1\"/>\n    <metadata key=\"limit_filament_maps\" value=\"0\"/>\n    <object identify_id=\"{}\" name=\"{}\" skipped=\"false\" />\n    <filament id=\"1\" tray_info_idx=\"{}\" type=\"{}\" color=\"{}\" used_m=\"{used_m:.2}\" used_g=\"{used_g:.2}\" group_id=\"0\" nozzle_diameter=\"{:.2}\" volume_type=\"Standard\" used_for_object=\"true\" used_for_support=\"false\"/>\n    <nozzle id=\"0\" extruder_id=\"1\" nozzle_diameter=\"{:.1}\" volume_type=\"Standard\"/>\n    <layer_filament_lists>\n      <layer_filament_list filament_list=\"0\" layer_ranges=\"0 {last_layer}\" />\n    </layer_filament_lists>\n  </plate>\n</config>\n",
        archive.nozzle_diameter,
        processed.result.initial_layer_time,
        archive.identify_id,
        archive.object_name,
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

fn number(value: f64) -> String {
    crate::options::format_number(value)
}
