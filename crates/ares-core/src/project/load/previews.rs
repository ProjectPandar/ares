use std::collections::BTreeSet;

use crate::SliceError;

use super::super::{
    PackagePath,
    content_types::{ContentTypes, PNG_CONTENT_TYPE},
    model_settings::ModelSettings,
    relationships::{
        COVER_THUMBNAIL_MIDDLE_RELATIONSHIP_TYPE, COVER_THUMBNAIL_SMALL_RELATIONSHIP_TYPE,
        Relationships, THUMBNAIL_RELATIONSHIP_TYPE,
    },
};

pub(super) fn validate_root_previews(
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

pub(super) fn validate_plate_previews(
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
