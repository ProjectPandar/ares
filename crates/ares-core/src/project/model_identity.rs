use serde::Deserialize;

use crate::SliceError;

use super::{
    PackagePath, ProjectArchive,
    model_xml::ModelMetadata,
    relationships::{MODEL_RELATIONSHIP_TYPE, Relationships},
    xml::{XmlRole, deserialize_xml},
};

#[derive(Deserialize)]
pub(super) struct ModelIdentity {
    #[serde(rename = "$value", default)]
    children: Vec<IdentityChild>,
}

#[derive(Deserialize)]
enum IdentityChild {
    #[serde(rename = "metadata")]
    Metadata(ModelMetadata),
    #[serde(other)]
    Other,
}

impl ModelIdentity {
    pub(super) fn read(archive: &mut ProjectArchive<'_>) -> Result<Self, SliceError> {
        let root = PackagePath::root();
        let relationships: Relationships = deserialize_xml(
            &archive.read(&root.resolve("_rels/.rels")?)?,
            XmlRole::Relationships,
        )?;
        let model_path = relationships.resolve_required(&root, MODEL_RELATIONSHIP_TYPE)?;
        deserialize_xml(&archive.read(&model_path)?, XmlRole::Model)
    }

    pub(super) fn metadata(&self) -> impl DoubleEndedIterator<Item = &ModelMetadata> {
        self.children.iter().filter_map(|child| match child {
            IdentityChild::Metadata(entry) => Some(entry),
            IdentityChild::Other => None,
        })
    }
}
