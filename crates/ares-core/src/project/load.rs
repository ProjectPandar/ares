mod assemble;
mod colors;
mod graph;
mod mesh_prepare;
mod metadata;
mod volume_metadata;

#[cfg(test)]
pub(crate) use volume_metadata::selected_volume_metadata_for_test;

use std::collections::BTreeSet;

use crate::{ORCA_SLICER_COMPATIBILITY_VERSION, ProjectSettings, SliceError};

use super::{
    ArchiveLimits, PackagePath, ProjectArchive,
    content_types::{
        ContentTypes, MODEL_CONTENT_TYPE, PNG_CONTENT_TYPE, RELATIONSHIPS_CONTENT_TYPE,
    },
    domain::{Project, ProjectDocuments},
    filament_sequence::FilamentSequences,
    model_settings::ModelSettings,
    plate::PlateJson,
    relationships::{
        COVER_THUMBNAIL_MIDDLE_RELATIONSHIP_TYPE, COVER_THUMBNAIL_SMALL_RELATIONSHIP_TYPE,
        MODEL_RELATIONSHIP_TYPE, Relationships, THUMBNAIL_RELATIONSHIP_TYPE,
    },
    slice_info::SliceInfo,
    xml::{JsonRole, XmlRole, deserialize_json, deserialize_xml},
};

const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";
const ROOT_RELATIONSHIPS_PATH: &str = "_rels/.rels";
const MODEL_SETTINGS_PATH: &str = "Metadata/model_settings.config";
const SLICE_INFO_PATH: &str = "Metadata/slice_info.config";
const FILAMENT_SEQUENCE_PATH: &str = "Metadata/filament_sequence.json";
const PROJECT_SETTINGS_PATH: &str = "Metadata/project_settings.config";
const LAYER_HEIGHT_PROFILE_PATH: &str = "Metadata/layer_heights_profile.txt";

pub fn load_project(input: impl AsRef<[u8]>) -> Result<Project, SliceError> {
    let input = input.as_ref();
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }

    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let archive_paths = archive.paths().cloned().collect::<BTreeSet<_>>();
    let has_painted_layer_height_profile = archive_paths.iter().any(|path| {
        path.as_str()
            .eq_ignore_ascii_case(LAYER_HEIGHT_PROFILE_PATH)
    });
    let content_types: ContentTypes =
        read_xml(&mut archive, CONTENT_TYPES_PATH, XmlRole::ContentTypes)?;
    content_types.validate_required()?;

    let root_relationships_path = PackagePath::entry(ROOT_RELATIONSHIPS_PATH.as_bytes())?;
    if content_types.content_type(&root_relationships_path) != Some(RELATIONSHIPS_CONTENT_TYPE) {
        return Err(SliceError::InvalidInput(
            "root relationship part has the wrong content type".to_owned(),
        ));
    }
    let root_relationships: Relationships = read_xml(
        &mut archive,
        ROOT_RELATIONSHIPS_PATH,
        XmlRole::Relationships,
    )?;
    root_relationships.validate_unique_ids(&PackagePath::root())?;
    let root_model =
        root_relationships.resolve_required(&PackagePath::root(), MODEL_RELATIONSHIP_TYPE)?;
    require_model_content_type(&content_types, &root_model)?;
    validate_root_previews(&root_relationships, &content_types, &archive_paths)?;

    let graph = graph::load(&mut archive, &content_types, &archive_paths, root_model)?;

    let model_settings: ModelSettings =
        read_xml(&mut archive, MODEL_SETTINGS_PATH, XmlRole::ModelSettings)?;
    validate_plate_previews(&model_settings, &content_types, &archive_paths)?;
    content_types.validate_png_entries(&mut archive)?;
    let slice_info: SliceInfo = read_xml(&mut archive, SLICE_INFO_PATH, XmlRole::SliceInfo)?;
    validate_compatibility(&slice_info)?;
    let filament_sequences: FilamentSequences = read_json(
        &mut archive,
        FILAMENT_SEQUENCE_PATH,
        JsonRole::FilamentSequences,
    )?;
    let metadata = metadata::index(&model_settings)?;
    metadata::validate_filament_plates(&metadata, &filament_sequences)?;

    let mut plate_documents = Vec::with_capacity(metadata.plates.len());
    for plate in &metadata.plates {
        let path = format!("Metadata/plate_{}.json", plate.id());
        // OrcaSlicer only writes plate documents when saving a sliced plate;
        // CLI exports of unsliced projects omit them.
        if archive_paths.contains(&PackagePath::root().resolve(&path)?) {
            let document: PlateJson = read_json(&mut archive, &path, JsonRole::Plate)?;
            metadata::validate_plate_document(&document)?;
            plate_documents.push(document);
        }
    }
    let settings: ProjectSettings = read_json(
        &mut archive,
        PROJECT_SETTINGS_PATH,
        JsonRole::ProjectSettings,
    )?;
    let project_settings_raw: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_slice(&read(&mut archive, PROJECT_SETTINGS_PATH)?).map_err(|error| {
            SliceError::InvalidInput(format!("invalid {PROJECT_SETTINGS_PATH}: {error}"))
        })?;

    let (models, mut objects) = assemble::project_domain(&graph, &metadata, &model_settings)?;
    super::layer_config_ranges::load(&mut archive, &archive_paths, &mut objects)?;
    Ok(Project::new(
        models,
        objects,
        metadata.plates,
        settings,
        ProjectDocuments {
            project_settings_raw,
            model_settings,
            slice_info,
            filament_sequences,
            plate_documents,
            has_painted_layer_height_profile,
        },
    ))
}

fn read(archive: &mut ProjectArchive<'_>, path: &str) -> Result<Vec<u8>, SliceError> {
    archive.read(&PackagePath::entry(path.as_bytes())?)
}

fn read_xml<T: serde::de::DeserializeOwned>(
    archive: &mut ProjectArchive<'_>,
    path: &str,
    role: XmlRole,
) -> Result<T, SliceError> {
    deserialize_xml(&read(archive, path)?, role)
}

fn read_json<T: serde::de::DeserializeOwned>(
    archive: &mut ProjectArchive<'_>,
    path: &str,
    role: JsonRole,
) -> Result<T, SliceError> {
    deserialize_json(&read(archive, path)?, role)
}

fn require_model_content_type(
    content_types: &ContentTypes,
    path: &PackagePath,
) -> Result<(), SliceError> {
    if content_types.content_type(path) != Some(MODEL_CONTENT_TYPE) {
        return Err(SliceError::InvalidInput(format!(
            "project model part {:?} has the wrong content type",
            path.as_str()
        )));
    }
    Ok(())
}

fn validate_compatibility(slice_info: &SliceInfo) -> Result<(), SliceError> {
    let version = slice_info
        .header
        .items
        .iter()
        .find(|item| item.key == "OrcaSlicer-Version")
        .map(|item| item.value.as_str());
    if version != Some(ORCA_SLICER_COMPATIBILITY_VERSION) {
        return Err(SliceError::InvalidInput(format!(
            "project requires OrcaSlicer compatibility version {ORCA_SLICER_COMPATIBILITY_VERSION}"
        )));
    }
    Ok(())
}

fn validate_root_previews(
    relationships: &Relationships,
    content_types: &ContentTypes,
    archive_paths: &BTreeSet<PackagePath>,
) -> Result<(), SliceError> {
    for relationship in relationships.relationships.iter().filter(|entry| {
        matches!(
            entry.relationship_type.as_str(),
            THUMBNAIL_RELATIONSHIP_TYPE
                | COVER_THUMBNAIL_MIDDLE_RELATIONSHIP_TYPE
                | COVER_THUMBNAIL_SMALL_RELATIONSHIP_TYPE
        )
    }) {
        let path = PackagePath::root().resolve(&relationship.target)?;
        validate_preview_path(&path, content_types, archive_paths)?;
    }
    Ok(())
}

fn validate_plate_previews(
    settings: &ModelSettings,
    content_types: &ContentTypes,
    archive_paths: &BTreeSet<PackagePath>,
) -> Result<(), SliceError> {
    for value in settings
        .plates
        .iter()
        .flat_map(|plate| &plate.metadata)
        .filter(|metadata| {
            matches!(
                metadata.key.as_str(),
                "thumbnail_file" | "thumbnail_no_light_file" | "top_file" | "pick_file"
            ) && !metadata.value.is_empty()
        })
        .map(|metadata| metadata.value.as_str())
    {
        let path = PackagePath::root().resolve(value)?;
        validate_preview_path(&path, content_types, archive_paths)?;
    }
    Ok(())
}

// OrcaSlicer writes thumbnail relationships unconditionally
// (`3mf.cpp _add_relationships_file_to_archive`) and ignores referenced
// preview parts missing from the archive; Ares only validates parts that
// exist.
fn validate_preview_path(
    path: &PackagePath,
    content_types: &ContentTypes,
    archive_paths: &BTreeSet<PackagePath>,
) -> Result<(), SliceError> {
    if !archive_paths.contains(path) {
        return Ok(());
    }
    if content_types.content_type(path) != Some(PNG_CONTENT_TYPE) {
        return Err(SliceError::InvalidInput(format!(
            "project preview {:?} has the wrong content type",
            path.as_str()
        )));
    }
    Ok(())
}
