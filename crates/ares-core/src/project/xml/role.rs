pub(super) const CONTENT_TYPES_NAMESPACE: &[u8] =
    b"http://schemas.openxmlformats.org/package/2006/content-types";
pub(super) const RELATIONSHIPS_NAMESPACE: &[u8] =
    b"http://schemas.openxmlformats.org/package/2006/relationships";
pub(super) const MODEL_NAMESPACE: &[u8] =
    b"http://schemas.microsoft.com/3dmanufacturing/core/2015/02";

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum XmlRole {
    ContentTypes,
    Relationships,
    Model,
    ModelSettings,
    SliceInfo,
}

impl XmlRole {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::ContentTypes => "content types",
            Self::Relationships => "relationships",
            Self::Model => "model",
            Self::ModelSettings => "model settings",
            Self::SliceInfo => "slice info",
        }
    }

    pub(super) fn root(self) -> &'static [u8] {
        match self {
            Self::ContentTypes => b"Types",
            Self::Relationships => b"Relationships",
            Self::Model => b"model",
            Self::ModelSettings | Self::SliceInfo => b"config",
        }
    }

    pub(super) fn namespace(self) -> Option<&'static [u8]> {
        match self {
            Self::ContentTypes => Some(CONTENT_TYPES_NAMESPACE),
            Self::Relationships => Some(RELATIONSHIPS_NAMESPACE),
            Self::Model => Some(MODEL_NAMESPACE),
            Self::ModelSettings | Self::SliceInfo => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum JsonRole {
    FilamentSequences,
    Plate,
}

impl JsonRole {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::FilamentSequences => "filament sequences",
            Self::Plate => "plate metadata",
        }
    }
}
