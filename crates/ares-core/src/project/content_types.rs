use serde::Deserialize;

use crate::SliceError;

use super::{PackagePath, ProjectArchive};

pub(crate) const MODEL_CONTENT_TYPE: &str =
    "application/vnd.ms-package.3dmanufacturing-3dmodel+xml";
pub(crate) const PNG_CONTENT_TYPE: &str = "image/png";
pub(crate) const RELATIONSHIPS_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-package.relationships+xml";

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Types")]
pub(crate) struct ContentTypes {
    #[serde(rename = "Default", default)]
    pub defaults: Vec<DefaultContentType>,
    #[serde(rename = "Override", default)]
    pub overrides: Vec<OverrideContentType>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct DefaultContentType {
    #[serde(rename = "@Extension")]
    pub extension: String,
    #[serde(rename = "@ContentType")]
    pub content_type: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct OverrideContentType {
    #[serde(rename = "@PartName")]
    pub part_name: String,
    #[serde(rename = "@ContentType")]
    pub content_type: String,
}

impl ContentTypes {
    pub(crate) fn validate_required(&self) -> Result<(), SliceError> {
        for entry in &self.overrides {
            PackagePath::entry(entry.part_name.as_bytes()).map_err(|error| {
                SliceError::InvalidInput(format!(
                    "project content types contain an invalid override path: {error}"
                ))
            })?;
        }
        for (extension, content_type) in [
            ("rels", RELATIONSHIPS_CONTENT_TYPE),
            ("model", MODEL_CONTENT_TYPE),
            ("png", PNG_CONTENT_TYPE),
        ] {
            if !self
                .defaults
                .iter()
                .any(|entry| entry.extension == extension && entry.content_type == content_type)
            {
                return Err(SliceError::InvalidInput(format!(
                    "project content types are missing {extension:?} as {content_type:?}"
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn content_type<'a>(&'a self, path: &PackagePath) -> Option<&'a str> {
        if let Some(content_type) = self.overrides.iter().find_map(|entry| {
            PackagePath::entry(entry.part_name.as_bytes())
                .ok()
                .filter(|override_path| override_path == path)
                .map(|_| entry.content_type.as_str())
        }) {
            return Some(content_type);
        }

        let extension = path.as_str().rsplit_once('.')?.1;
        self.defaults
            .iter()
            .find(|entry| entry.extension.eq_ignore_ascii_case(extension))
            .map(|entry| entry.content_type.as_str())
    }

    pub(crate) fn validate_png_entries(
        &self,
        archive: &mut ProjectArchive<'_>,
    ) -> Result<Vec<PackagePath>, SliceError> {
        let paths = archive
            .paths()
            .filter(|path| self.content_type(path) == Some(PNG_CONTENT_TYPE))
            .cloned()
            .collect::<Vec<_>>();
        for path in &paths {
            drop(archive.read(path)?);
        }
        Ok(paths)
    }
}
