mod assemble;
mod colors;
mod graph;
mod mesh_prepare;
mod metadata;
mod volume_metadata;

use std::collections::BTreeSet;

use crate::SliceError;

use super::{
    ArchiveLimits, PackagePath, ProjectArchive,
    content_types::{
        ContentTypes, MODEL_CONTENT_TYPE, PNG_CONTENT_TYPE, RELATIONSHIPS_CONTENT_TYPE,
    },
    domain::{Project, ProjectDocuments},
    filament_sequence::FilamentSequences,
    model_settings::ModelSettings,
    plate::PlateJson,
    raw_settings::ProjectSettingsRaw,
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

/// One value the 3MF load replaced (`ConfigSubstitution`, `Config.hpp:263-267`): the option key,
/// the wire text it carried and the value it now holds, serialized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigSubstitution {
    pub key: String,
    pub old_value: String,
    pub new_value: String,
}

impl ConfigSubstitution {
    pub(crate) fn new(key: &str, old_value: &str, new_value: &str) -> Self {
        Self {
            key: key.to_owned(),
            old_value: old_value.to_owned(),
            new_value: new_value.to_owned(),
        }
    }
}

/// The substitutions the 3MF load makes in the project settings
/// (`config_substitutions` of `Model::read_from_file`, `OrcaSlicer.cpp:1737-1758`), which the CLI
/// reports after the load (`cli_report_config_substitutions`, `:1992`). `load_from_json` hands
/// each value to `handle_legacy` before `set_deserialize_raw` substitutes it (`set_deserialize_nothrow`, `Config.cpp:596-611`), so
/// a legacy token (`enable_arc_fitting` "0") is translated, not substituted.
pub fn project_config_substitutions(
    input: impl AsRef<[u8]>,
) -> Result<Vec<ConfigSubstitution>, SliceError> {
    let mut archive = ProjectArchive::open(input.as_ref(), ArchiveLimits::PROJECT)?;
    let mut raw: ProjectSettingsRaw =
        serde_json::from_slice(&read(&mut archive, PROJECT_SETTINGS_PATH)?).map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    raw.drop_invalid_wire_scalars();
    raw.handle_legacy();
    Ok(raw.normalize_embedded_wire_substitutions())
}

pub fn load_project(input: impl AsRef<[u8]>) -> Result<Project, SliceError> {
    load_project_with(input, &crate::ConfigOverrides::default())
}

/// `load_project` with user option overrides applied on the
/// project-settings layer (`ConfigBase::apply_only`, `Config.cpp:461`:
/// last write wins). See `crate::ConfigOverrides` (M854).
pub fn load_project_with(
    input: impl AsRef<[u8]>,
    overrides: &crate::ConfigOverrides,
) -> Result<Project, SliceError> {
    load_project_with_preparation(input, overrides).map(|(project, _)| project)
}

pub(crate) fn load_project_with_preparation(
    input: impl AsRef<[u8]>,
    overrides: &crate::ConfigOverrides,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    load_project_raw(input.as_ref(), overrides, true)
}

pub(crate) fn load_project_for_profiles(
    input: &[u8],
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
    load_project_raw(input, &crate::ConfigOverrides::default(), false)
}

fn load_project_raw(
    input: &[u8],
    overrides: &crate::ConfigOverrides,
    materialize_retained_defaults: bool,
) -> Result<(Project, crate::options::flush_matrix::PreparationContext), SliceError> {
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
    let settings_bytes = read(&mut archive, PROJECT_SETTINGS_PATH)?;
    let mut project_settings_raw: ProjectSettingsRaw = serde_json::from_slice(&settings_bytes)
        .map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    // `load_from_json`'s type dispatch runs before any per-key
    // deserialization: non-string/non-array JSON scalars never reach
    // `set_deserialize` and the key keeps its default (`Config.cpp:905-990`).
    project_settings_raw.drop_invalid_wire_scalars();
    // `Config.cpp:596-617` (345efec64c7b): legacy rewriting precedes
    // `set_deserialize_raw`'s substitution at 680-695.
    project_settings_raw.handle_legacy();
    // The 3MF load substitutes unknown enum/bool wires on the legacy-normalized
    // EMBEDDED config (the CLI passes `Enable`, `OrcaSlicer.cpp:1380, 1737`, and
    // `project_config_substitutions` reports them). Caller overrides below merge
    // through their own deserialization rule and are not substituted.
    project_settings_raw.normalize_embedded_wire_substitutions();
    // `handle_legacy_composite` (`PrintConfig.cpp:10094-10096`) ends the same `load_from_json`.
    project_settings_raw.normalize_filament_values_to_variants();
    project_settings_raw
        .normalize_loaded_composites()
        .map_err(SliceError::InvalidInput)?;
    // CLI:1704 captures the old configuration before user edits.
    let preparation = crate::options::flush_matrix::PreparationContext::capture(
        &project_settings_raw,
        overrides,
    )?;
    // Preset.cpp:1059-1089 constructs retained presets over their full defaults,
    // excluding embedded connection credentials from the project overlay.
    if materialize_retained_defaults {
        crate::profiles::apply_retained_preset_defaults(&mut project_settings_raw, None)?;
    }
    // Overrides win over document values (`apply_only`, `Config.cpp:461`).
    overrides.apply(&mut project_settings_raw)?;
    // Same upstream deserializer semantics as preset documents: JSON
    // strings on vector-kind options deserialize through
    // `ConfigOptionVector::deserialize` (`Config.hpp:868-887`,
    // `ConfigOptionStrings::deserialize`, `Config.hpp:1153-1159`) — a bare
    // scalar is a single-element vector, an empty string an empty one.
    project_settings_raw
        .normalize_wire_vector_shapes()
        .map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    // Caller overrides are a later serialized write (`Config.cpp:602`);
    // normalize their tokens without repeating an embedded-only load pass.
    if !overrides.is_empty() {
        project_settings_raw.handle_legacy();
    }
    // The array branch of the same upstream load: joined string arrays bind
    // scalar options (`Config.cpp:945-1001`), so the validation flat map and
    // the typed builders see the bound scalar, not the array.
    project_settings_raw
        .normalize_scalar_arrays()
        .map_err(|error| {
            SliceError::InvalidInput(format!("invalid project settings JSON: {error}"))
        })?;
    let mut materialized_settings_raw = project_settings_raw.clone();
    materialized_settings_raw.complete_filament_variant_defaults()?;
    let settings = <crate::options::ProjectSettingsBuilder as serde::Deserialize>::deserialize(
        materialized_settings_raw.into_object(),
    )
    .map_err(|error| SliceError::InvalidInput(format!("invalid project settings JSON: {error}")))?
    .resolve();

    let (models, mut objects) = assemble::project_domain(&graph, &metadata, &model_settings)?;
    super::domain::materialize_meshes(&mut objects);
    super::domain::normalize_volume_offsets(&mut objects);
    super::layer_config_ranges::load(&mut archive, &archive_paths, &mut objects)?;
    super::layer_height_profiles::load(&mut archive, &archive_paths, &mut objects)?;
    super::brim_ear_points::load(&mut archive, &archive_paths, &mut objects)?;
    let plates_custom_gcodes = super::plate_custom_gcodes::load(&mut archive, &archive_paths)?;
    // bbs_3mf.cpp:3945-3957: the model's `Application` metadata tags the
    // BambuStudio/OrcaSlicer family (`is_bbl_3mf`).
    let bbl_family_3mf = graph.models.iter().any(|model| {
        model.document.metadata.iter().any(|entry| {
            entry.name == "Application"
                && (entry.value.starts_with("BambuStudio-")
                    || entry.value.starts_with("OrcaSlicer-"))
        })
    });
    // bbs_3mf.cpp:1539-1551: OrcaSlicer metadata takes precedence over Application.
    let preset_file_version = graph
        .models
        .iter()
        .flat_map(|model| &model.document.metadata)
        .rfind(|entry| entry.name == "OrcaSlicer")
        .and_then(|entry| super::preset_version::parse(&entry.value))
        .or_else(|| {
            graph
                .models
                .iter()
                .flat_map(|model| &model.document.metadata)
                .rev()
                .filter(|entry| entry.name == "Application")
                .find_map(|entry| {
                    entry
                        .value
                        .strip_prefix("OrcaSlicer-")
                        .or_else(|| entry.value.strip_prefix("BambuStudio-"))
                })
                .and_then(super::preset_version::parse)
        });
    let mut project = Project::new(
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
            bbl_family_3mf,
            preset_file_version,
            preset_selection: None,
        },
    );
    project.plates_custom_gcodes = plates_custom_gcodes;
    Ok((project, preparation))
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
