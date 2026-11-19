use std::fmt::Write;

use super::{BAMBU_VERSION, geometry::Mesh};
use crate::ORCA_SLICER_COMPATIBILITY_VERSION as ORCA_VERSION;

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
    let path = quick_xml::escape::escape(path);
    format!(
        "{DECL}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n <Relationship Target=\"/{path}\" Id=\"rel-1\" Type=\"{}\"/>\n</Relationships>",
        super::super::relationships::MODEL_RELATIONSHIP_TYPE
    )
}

pub(super) fn main_model(mesh: &Mesh, part: &str) -> String {
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

pub(super) fn model_settings(mesh: &Mesh, filename: &str) -> String {
    let name = quick_xml::escape::escape(&mesh.name);
    let filename = quick_xml::escape::escape(filename);
    let [x, y, z] = mesh.source_offset.map(f64::from).map(number);
    format!(
        "{DECL}<config>\n  <object id=\"2\" auto_drop=\"1\">\n    <metadata key=\"name\" value=\"{name}\"/>\n    <metadata key=\"extruder\" value=\"1\"/>\n    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"name\" value=\"{name}\"/>\n      <metadata key=\"matrix\" value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"/>\n      <metadata key=\"source_file\" value=\"{filename}\"/>\n      <metadata key=\"source_object_id\" value=\"0\"/>\n      <metadata key=\"source_volume_id\" value=\"0\"/>\n      <metadata key=\"source_offset_x\" value=\"{x}\"/>\n      <metadata key=\"source_offset_y\" value=\"{y}\"/>\n      <metadata key=\"source_offset_z\" value=\"{z}\"/>\n      <mesh_stat edges_fixed=\"0\" degenerate_facets=\"0\" facets_removed=\"0\" facets_reversed=\"0\" backwards_edges=\"0\"/>\n    </part>\n  </object>\n  <plate>\n    <metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"\"/>\n  </plate>\n  <assemble>\n  </assemble>\n</config>\n"
    )
}

pub(super) fn slice_info() -> String {
    format!(
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n</config>\n"
    )
}

fn number(value: f64) -> String {
    crate::options::format_number(value)
}
