use std::collections::BTreeSet;

use serde::Deserialize;

use crate::SliceError;

use super::PackagePath;

pub(crate) const MODEL_RELATIONSHIP_TYPE: &str =
    "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";
pub(crate) const THUMBNAIL_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";
pub(crate) const COVER_THUMBNAIL_MIDDLE_RELATIONSHIP_TYPE: &str =
    "http://schemas.bambulab.com/package/2021/cover-thumbnail-middle";
pub(crate) const COVER_THUMBNAIL_SMALL_RELATIONSHIP_TYPE: &str =
    "http://schemas.bambulab.com/package/2021/cover-thumbnail-small";

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Relationships")]
pub(crate) struct Relationships {
    #[serde(rename = "Relationship", default)]
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Relationship {
    #[serde(rename = "@Target")]
    pub target: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Type")]
    pub relationship_type: String,
}

impl Relationships {
    pub(crate) fn validate_unique_ids(&self, owner: &PackagePath) -> Result<(), SliceError> {
        let owner = if owner.as_str().is_empty() {
            "package root"
        } else {
            owner.as_str()
        };
        let mut ids = BTreeSet::new();
        for relationship in &self.relationships {
            if !ids.insert(relationship.id.as_str()) {
                return Err(SliceError::InvalidInput(format!(
                    "project relationships owned by {owner:?} contain duplicate relationship ID {:?}",
                    relationship.id
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn resolve_required(
        &self,
        owner: &PackagePath,
        relationship_type: &str,
    ) -> Result<PackagePath, SliceError> {
        let mut matching = self
            .relationships
            .iter()
            .filter(|relationship| relationship.relationship_type == relationship_type);
        let relationship = matching.next().ok_or_else(|| {
            SliceError::InvalidInput(format!(
                "project relationships for {:?} are missing required type {relationship_type:?}",
                owner.as_str()
            ))
        })?;
        if matching.next().is_some() {
            return Err(SliceError::InvalidInput(format!(
                "project relationships for {:?} repeat required type {relationship_type:?}",
                owner.as_str()
            )));
        }
        owner.resolve(&relationship.target)
    }
}
