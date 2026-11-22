use std::fmt::Write;

use super::{BAMBU_VERSION, ObjectEntry, SlicedMembers, scene::SceneLayout};
use crate::ORCA_SLICER_COMPATIBILITY_VERSION as ORCA_VERSION;
use crate::project::model_settings::Metadata;
use crate::{ProcessedGCode, SlicedArchive, Transform3d};

mod plate;

pub(super) use plate::plate_json;

const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
const MODEL_OPEN: &str = concat!(
    "<model unit=\"millimeter\" xml:lang=\"en-US\" ",
    "xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" ",
    "xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" ",
    "xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n"
);
const VERSION_METADATA: &str = " <metadata name=\"BambuStudio:3mfVersion\">1</metadata>\n";
fn transform_3mf(transform: Transform3d, use_f32: bool) -> String {
    let column_major = transform.to_column_major();
    let values = column_major
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|column| &column[..3])
        .copied();
    if use_f32 {
        numbers_f32(values)
    } else {
        numbers(values)
    }
}

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

pub(super) fn gcode_relationship<'a>(paths: impl IntoIterator<Item = &'a str>) -> String {
    relationship(paths, "http://schemas.bambulab.com/package/2021/gcode")
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
            let transform = transform_3mf(part.transform, part.use_f32_transform);
            writeln!(out, "    <component p:path=\"/{path}\" objectid=\"{id}\" p:UUID=\"{prefix:08x}-b206-40ff-9872-83e8017abed1\" transform=\"{transform}\"/>").unwrap();
        }
        out.push_str("   </components>\n  </object>\n");
    }
    out.push_str(" </resources>\n <build p:UUID=\"2c7c17d8-22b5-4d84-8835-1976022ea369\">\n");
    for object in objects {
        let id = object.id;
        for instance in &object.object.instances {
            let transform = transform_3mf(instance.transform, false);
            let printable = u8::from(instance.printable);
            writeln!(out, "  <item objectid=\"{id}\" p:UUID=\"{id:08x}-b1ec-4553-aec9-835e5b724bb4\" transform=\"{transform}\" printable=\"{printable}\"/>").unwrap();
        }
    }
    out.push_str(" </build>\n</model>\n");
    out
}

pub(super) fn mesh_model(object: &ObjectEntry<'_>) -> String {
    let mut out = format!("{DECL}{MODEL_OPEN}{VERSION_METADATA} <resources>\n");
    for (id, prefix, part) in object.parts() {
        writeln!(out, "  <object id=\"{id}\" p:UUID=\"{prefix:08x}-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <mesh>\n    <vertices>").unwrap();
        for [x, y, z] in &part.mesh.vertices {
            if part.mesh.use_f32_vertices {
                writeln!(
                    out,
                    "     <vertex x=\"{}\" y=\"{}\" z=\"{}\"/>",
                    f32_number(*x),
                    f32_number(*y),
                    f32_number(*z)
                )
                .unwrap();
            } else {
                writeln!(out, "     <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
            }
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

pub(super) fn metadata(out: &mut String, indent: &str, key: &str, value: &str) {
    let key = quick_xml::escape::escape(key);
    let value = quick_xml::escape::escape(value);
    writeln!(out, "{indent}<metadata key=\"{key}\" value=\"{value}\"/>").unwrap();
}

fn write_metadata(out: &mut String, indent: &str, records: &[Metadata]) {
    for Metadata { key, value } in records {
        metadata(out, indent, key, value);
    }
}

pub(super) fn model_settings(
    objects: &[ObjectEntry<'_>],
    sliced: Option<&[SlicedMembers]>,
    saved_instance_id: Option<u32>,
    scene: Option<&SceneLayout<'_>>,
) -> String {
    let mut out = format!("{DECL}<config>\n");
    for object in objects {
        let id = object.id;
        let auto_drop = u8::from(object.object.auto_drop);
        writeln!(out, "  <object id=\"{id}\" auto_drop=\"{auto_drop}\">").unwrap();
        metadata(&mut out, "    ", "name", &object.object.name);
        write_metadata(&mut out, "    ", &object.object.metadata);
        for (id, _, part) in object.parts() {
            let name = quick_xml::escape::escape(&part.mesh.name);
            let filename = quick_xml::escape::escape(&part.source_file);
            let matrix = if part.mesh.use_f32_vertices {
                full_numbers(part.matrix.as_matrix().iter().flatten().copied())
            } else {
                numbers(part.matrix.as_matrix().iter().flatten().copied())
            };
            let [x, y, z] = if part.mesh.use_f32_vertices {
                part.mesh.source_offset.map(f32_full_number)
            } else {
                part.mesh.source_offset.map(f64::from).map(number)
            };
            let (source_object, source_volume) = (part.source_object, part.source_volume);
            writeln!(out, "    <part id=\"{id}\" subtype=\"normal_part\">\n      <metadata key=\"name\" value=\"{name}\"/>\n      <metadata key=\"matrix\" value=\"{matrix}\"/>").unwrap();
            if !filename.is_empty() {
                writeln!(out, "      <metadata key=\"source_file\" value=\"{filename}\"/>\n      <metadata key=\"source_object_id\" value=\"{source_object}\"/>\n      <metadata key=\"source_volume_id\" value=\"{source_volume}\"/>\n      <metadata key=\"source_offset_x\" value=\"{x}\"/>\n      <metadata key=\"source_offset_y\" value=\"{y}\"/>\n      <metadata key=\"source_offset_z\" value=\"{z}\"/>").unwrap();
            }
            write_metadata(&mut out, "      ", &part.metadata);
            let stats = part.mesh_stat.as_ref();
            let edges = stats.map_or(0, |stat| stat.edges_fixed);
            let degenerate = stats.map_or(0, |stat| stat.degenerate_facets);
            let removed = stats.map_or(0, |stat| stat.facets_removed);
            let reversed = stats.map_or(0, |stat| stat.facets_reversed);
            let backwards = stats.map_or(0, |stat| stat.backwards_edges);
            writeln!(out, "      <mesh_stat edges_fixed=\"{edges}\" degenerate_facets=\"{degenerate}\" facets_removed=\"{removed}\" facets_reversed=\"{reversed}\" backwards_edges=\"{backwards}\"/>\n    </part>").unwrap();
        }
        out.push_str("  </object>\n");
    }
    if let Some(scene) = scene {
        scene.write_model_settings(&mut out, objects);
        out.push_str("</config>\n");
        return out;
    }
    let plates = sliced.map_or_else(|| vec![None], |plates| plates.iter().map(Some).collect());
    for (index, plate) in plates.into_iter().enumerate() {
        let plate_id = index + 1;
        let gcode_file = plate
            .map(|_| format!("Metadata/plate_{plate_id}.gcode"))
            .unwrap_or_default();
        writeln!(out, "  <plate>\n    <metadata key=\"plater_id\" value=\"{plate_id}\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"{gcode_file}\"/>").unwrap();
        let saved_object = saved_instance_id.map(|id| (0, id));
        let object_ids = plate.map_or(saved_object.as_slice(), |plate| plate.object_ids.as_slice());
        for &(model_index, identify_id) in object_ids {
            let id = objects[model_index].id;
            writeln!(out, "    <model_instance>\n      <metadata key=\"object_id\" value=\"{id}\"/>\n      <metadata key=\"instance_id\" value=\"0\"/>\n      <metadata key=\"identify_id\" value=\"{identify_id}\"/>\n    </model_instance>").unwrap();
        }
        out.push_str("  </plate>\n");
    }
    out.push_str("  <assemble>\n  </assemble>\n</config>\n");
    out
}

pub(super) fn slice_info(
    archive: &SlicedArchive,
    processed: &ProcessedGCode,
    plate: u32,
) -> String {
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
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n  <plate>\n    <metadata key=\"index\" value=\"{plate}\"/>\n    <metadata key=\"extruder_type\" value=\"0\"/>\n    <metadata key=\"nozzle_volume_type\" value=\"0\"/>\n    <metadata key=\"printer_model_id\" value=\"\"/>\n    <metadata key=\"nozzle_diameters\" value=\"{:.1}\"/>\n    <metadata key=\"timelapse_type\" value=\"0\"/>\n    <metadata key=\"prediction\" value=\"{prediction}\"/>\n    <metadata key=\"weight\" value=\"{weight:.2}\"/>\n    <metadata key=\"first_layer_time\" value=\"{first_layer_time}\"/>\n    <metadata key=\"outside\" value=\"false\"/>\n    <metadata key=\"support_used\" value=\"false\"/>\n    <metadata key=\"label_object_enabled\" value=\"true\"/>\n    <metadata key=\"enable_filament_dynamic_map\" value=\"false\"/>\n    <metadata key=\"has_filament_switcher\" value=\"false\"/>\n    <metadata key=\"filament_maps\" value=\"1\"/>\n    <metadata key=\"limit_filament_maps\" value=\"0\"/>\n{object_entries}    <filament id=\"1\" tray_info_idx=\"{}\" type=\"{}\" color=\"{}\" used_m=\"{used_m:.2}\" used_g=\"{used_g:.2}\" group_id=\"0\" nozzle_diameter=\"{:.2}\" volume_type=\"Standard\" used_for_object=\"true\" used_for_support=\"false\"/>\n    <nozzle id=\"0\" extruder_id=\"1\" nozzle_diameter=\"{:.1}\" volume_type=\"Standard\"/>\n    <layer_filament_lists>\n      <layer_filament_list filament_list=\"0\" layer_ranges=\"0 {last_layer}\" />\n    </layer_filament_lists>\n  </plate>\n</config>\n",
        archive.nozzle_diameter,
        archive.filament_id,
        archive.filament_type,
        archive.filament_color,
        archive.nozzle_diameter,
        archive.nozzle_diameter,
    )
}

pub(super) fn filament_sequence(plate_count: usize) -> Vec<u8> {
    let mut result = String::from("{");
    for plate in 1..=plate_count {
        if plate > 1 {
            result.push(',');
        }
        write!(
            result,
            "\"plate_{plate}\":{{\"nozzle_sequence\":[0],\"optimal_assignment\":[0],\"sequence\":[1]}}"
        )
        .unwrap();
    }
    result.push('}');
    result.into_bytes()
}

pub(super) fn slice_info_many(members: &[SlicedMembers]) -> String {
    if members.len() == 1 {
        return String::from_utf8(members[0].slice_info.clone()).unwrap();
    }
    let mut result = String::new();
    let first = String::from_utf8(members[0].slice_info.clone()).unwrap();
    let header_end = first.find("  <plate>\n").unwrap();
    result.push_str(&first[..header_end]);
    for member in members {
        let info = String::from_utf8(member.slice_info.clone()).unwrap();
        let start = info.find("  <plate>\n").unwrap();
        let end = info.rfind("</config>\n").unwrap();
        result.push_str(&info[start..end]);
    }
    result.push_str("</config>\n");
    result
}

pub(super) fn slice_info_header() -> String {
    format!(
        "{DECL}<config>\n  <header>\n    <header_item key=\"X-BBL-Client-Type\" value=\"slicer\"/>\n    <header_item key=\"X-BBL-Client-Version\" value=\"{BAMBU_VERSION}\"/>\n    <header_item key=\"OrcaSlicer-Version\" value=\"{ORCA_VERSION}\"/>\n  </header>\n</config>\n"
    )
}

fn numbers(values: impl Iterator<Item = f64>) -> String {
    values.map(number).collect::<Vec<_>>().join(" ")
}

fn full_numbers(values: impl Iterator<Item = f64>) -> String {
    values.map(full_number).collect::<Vec<_>>().join(" ")
}

fn numbers_f32(values: impl Iterator<Item = f64>) -> String {
    values
        .map(|value| f32_number(value as f32))
        .collect::<Vec<_>>()
        .join(" ")
}

fn f32_number(value: f32) -> String {
    let mut output = format!("{value:.9}");
    while output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}

fn f32_full_number(value: f32) -> String {
    full_number(f64::from(value))
}

fn full_number(value: f64) -> String {
    let mut output = format!("{value:.17}");
    while output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}

fn number(value: f64) -> String {
    crate::options::format_number(value)
}
