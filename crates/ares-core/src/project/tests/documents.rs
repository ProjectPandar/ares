use std::io::{Cursor, Write};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::project::{
    ArchiveLimits, PackagePath, ProjectArchive,
    content_types::{ContentTypes, MODEL_CONTENT_TYPE, PNG_CONTENT_TYPE},
    filament_sequence::FilamentSequences,
    model_settings::{Metadata, ModelSettings},
    plate::PlateJson,
    relationships::{MODEL_RELATIONSHIP_TYPE, Relationships, THUMBNAIL_RELATIONSHIP_TYPE},
    slice_info::SliceInfo,
    xml::{JsonRole, XmlRole, deserialize_json, deserialize_xml},
};

const FIXTURE: &[u8] =
    include_bytes!("../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");

#[test]
fn project_documents_deserialize_content_types_relationships_and_all_previews() {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let content_types: ContentTypes = deserialize_xml(
        &read(&mut archive, "[Content_Types].xml"),
        XmlRole::ContentTypes,
    )
    .unwrap();

    assert_eq!(content_types.defaults.len(), 4);
    assert_eq!(content_types.defaults[0].extension, "rels");
    assert_eq!(
        content_types.defaults[0].content_type,
        "application/vnd.openxmlformats-package.relationships+xml"
    );
    assert_eq!(content_types.defaults[1].extension, "model");
    assert_eq!(content_types.defaults[1].content_type, MODEL_CONTENT_TYPE);
    assert_eq!(content_types.defaults[2].extension, "png");
    assert_eq!(content_types.defaults[2].content_type, PNG_CONTENT_TYPE);
    assert!(content_types.overrides.is_empty());
    content_types.validate_required().unwrap();

    let root_relationships: Relationships =
        deserialize_xml(&read(&mut archive, "_rels/.rels"), XmlRole::Relationships).unwrap();
    assert_eq!(root_relationships.relationships.len(), 4);
    assert_eq!(root_relationships.relationships[0].id, "rel-1");
    assert_eq!(
        root_relationships.relationships[0].relationship_type,
        MODEL_RELATIONSHIP_TYPE
    );
    assert_eq!(
        root_relationships.relationships[0].target,
        "/3D/3dmodel.model"
    );

    let root_owner = PackagePath::root();
    let root_model = root_relationships
        .resolve_required(&root_owner, MODEL_RELATIONSHIP_TYPE)
        .unwrap();
    assert_eq!(root_model.as_str(), "3D/3dmodel.model");
    assert_eq!(
        content_types.content_type(&root_model),
        Some(MODEL_CONTENT_TYPE)
    );

    let thumbnail = root_relationships
        .resolve_required(&root_owner, THUMBNAIL_RELATIONSHIP_TYPE)
        .unwrap();
    assert_eq!(thumbnail.as_str(), "Metadata/plate_1.png");
    assert_eq!(
        content_types.content_type(&thumbnail),
        Some(PNG_CONTENT_TYPE)
    );

    let model_relationships: Relationships = deserialize_xml(
        &read(&mut archive, "3D/_rels/3dmodel.model.rels"),
        XmlRole::Relationships,
    )
    .unwrap();
    assert_eq!(model_relationships.relationships.len(), 1);
    let object_model = model_relationships
        .resolve_required(&root_model, MODEL_RELATIONSHIP_TYPE)
        .unwrap();
    assert_eq!(
        object_model.as_str(),
        "3D/Objects/ksr_fdmtest_v4.drc_2.model"
    );
    assert_eq!(
        content_types.content_type(&object_model),
        Some(MODEL_CONTENT_TYPE)
    );

    let preview_paths = content_types.validate_png_entries(&mut archive).unwrap();
    assert_eq!(
        preview_paths
            .iter()
            .map(PackagePath::as_str)
            .collect::<Vec<_>>(),
        [
            "Metadata/pick_1.png",
            "Metadata/plate_1.png",
            "Metadata/plate_1_small.png",
            "Metadata/plate_no_light_1.png",
            "Metadata/top_1.png",
        ]
    );
}

#[test]
fn project_documents_deserialize_typed_project_metadata() {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let model: ModelSettings = deserialize_xml(
        &read(&mut archive, "Metadata/model_settings.config"),
        XmlRole::ModelSettings,
    )
    .unwrap();

    assert_eq!(model.objects.len(), 1);
    let object = &model.objects[0];
    assert_eq!(object.id, 2);
    assert_eq!(metadata(&object.metadata, "name"), "ksr_fdmtest_v4.drc");
    assert_eq!(metadata(&object.metadata, "extruder"), "1");
    assert_eq!(object.parts.len(), 1);
    let part = &object.parts[0];
    assert_eq!(part.id, 1);
    assert_eq!(part.subtype, "normal_part");
    assert_eq!(
        metadata(&part.metadata, "matrix"),
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"
    );
    assert_eq!(part.mesh_stat.as_ref().unwrap().edges_fixed, 0);

    assert_eq!(model.plates.len(), 1);
    let plate = &model.plates[0];
    assert_eq!(metadata(&plate.metadata, "plater_id"), "1");
    assert_eq!(plate.model_instances.len(), 1);
    let instance = &plate.model_instances[0];
    assert_eq!(metadata(&instance.metadata, "object_id"), "2");
    assert_eq!(metadata(&instance.metadata, "identify_id"), "133");
    let assemble = model.assemble.as_ref().unwrap();
    assert_eq!(assemble.items.len(), 1);
    assert_eq!(assemble.items[0].object_id, 2);
    assert_eq!(assemble.items[0].instance_id, 0);
    assert_eq!(assemble.items[0].transform, "1 0 0 0 1 0 0 0 1 0 0 46");

    let slice_info: SliceInfo = deserialize_xml(
        &read(&mut archive, "Metadata/slice_info.config"),
        XmlRole::SliceInfo,
    )
    .unwrap();
    assert_eq!(
        slice_info
            .header
            .items
            .iter()
            .find(|item| item.key == "OrcaSlicer-Version")
            .unwrap()
            .value,
        "2.4.2"
    );

    let sequences: FilamentSequences = deserialize_json(
        &read(&mut archive, "Metadata/filament_sequence.json"),
        JsonRole::FilamentSequences,
    )
    .unwrap();
    assert_eq!(sequences.0.len(), 1);
    let (plate_id, sequence) = sequences.0.first_key_value().unwrap();
    assert_eq!(plate_id.get(), 1);
    assert!(sequence.sequence.is_empty());
    assert!(sequence.nozzle_sequence.is_empty());
    assert!(sequence.optimal_assignment.is_empty());

    let plate_json: PlateJson = deserialize_json(
        &read(&mut archive, "Metadata/plate_1.json"),
        JsonRole::Plate,
    )
    .unwrap();
    assert_eq!(
        plate_json.bbox_all,
        [95.539205, 81.892104, 169.639205, 150.99210399999998]
    );
    assert_eq!(plate_json.bbox_objects.len(), 1);
    assert_eq!(plate_json.bbox_objects[0].id, 147);
    assert_eq!(plate_json.bbox_objects[0].area, 2074.176025390625);
    assert_eq!(plate_json.first_layer_time, 456.104248046875);
    assert_eq!(plate_json.nozzle_diameter, 0.4000000059604645);
    assert_eq!(plate_json.version, 2);
}

#[test]
fn project_documents_resolve_root_and_part_relative_relationships() {
    let root: Relationships = deserialize_xml(
        br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="3D/model.model" Id="r1" Type="model"/></Relationships>"#,
        XmlRole::Relationships,
    )
    .unwrap();
    assert_eq!(
        root.resolve_required(&PackagePath::root(), "model")
            .unwrap()
            .as_str(),
        "3D/model.model"
    );

    let nested: Relationships = deserialize_xml(
        br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="Objects/object.model" Id="r1" Type="model"/></Relationships>"#,
        XmlRole::Relationships,
    )
    .unwrap();
    assert_eq!(
        nested
            .resolve_required(&PackagePath::entry(b"3D/model.model").unwrap(), "model")
            .unwrap()
            .as_str(),
        "3D/Objects/object.model"
    );
}

#[test]
fn project_documents_reject_malformed_content_type_override() {
    let content_types: ContentTypes = deserialize_xml(
        br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/><Override PartName="/../bad.png" ContentType="image/png"/></Types>"#,
        XmlRole::ContentTypes,
    )
    .unwrap();

    assert!(content_types.validate_required().is_err());
}

#[test]
fn project_documents_validate_unreferenced_preview_crc() {
    let bytes = archive_with_corrupt_unreferenced_preview();
    let mut archive = ProjectArchive::open(&bytes, ArchiveLimits::PROJECT).unwrap();
    let content_types: ContentTypes = deserialize_xml(
        &read(&mut archive, "[Content_Types].xml"),
        XmlRole::ContentTypes,
    )
    .unwrap();

    assert!(content_types.validate_png_entries(&mut archive).is_err());
}

fn metadata<'a>(entries: &'a [Metadata], key: &str) -> &'a str {
    &entries.iter().find(|entry| entry.key == key).unwrap().value
}

fn read(archive: &mut ProjectArchive<'_>, path: &str) -> Vec<u8> {
    archive
        .read(&PackagePath::entry(path.as_bytes()).unwrap())
        .unwrap()
}

fn archive_with_corrupt_unreferenced_preview() -> Vec<u8> {
    const CONTENT_TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="png" ContentType="image/png"/></Types>"#;
    const PREVIEW: &[u8] = b"opaque-unreferenced-preview-payload";
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    writer.start_file("[Content_Types].xml", options).unwrap();
    writer.write_all(CONTENT_TYPES).unwrap();
    writer
        .start_file("Metadata/unreferenced.png", options)
        .unwrap();
    writer.write_all(PREVIEW).unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    let offset = bytes
        .windows(PREVIEW.len())
        .position(|window| window == PREVIEW)
        .unwrap();
    bytes[offset] ^= 0xff;
    bytes
}
