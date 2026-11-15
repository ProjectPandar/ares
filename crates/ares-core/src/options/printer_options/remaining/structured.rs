use serde::{Deserialize, Serialize};

use super::super::super::config_types::{Nullable, OrcaFloat, semantic};

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct NullableFloats(pub Vec<Nullable<OrcaFloat>>);

impl Serialize for NullableFloats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        semantic::serialize_nullable_vector(&self.0, serializer)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct ExtruderVariantLists(pub Vec<String>);

impl Serialize for ExtruderVariantLists {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        semantic::serialize_string_vector(&self.0, serializer)
    }
}

macro_rules! semantic_string {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
            #[serde(transparent)]
            pub struct $name(pub String);
        )+
    };
}

semantic_string!(DefaultBedType, PrinterModel, PrinterNotes);

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ThumbnailDefinitions(pub super::super::super::config_types::OrcaString);

impl ThumbnailDefinitions {
    pub fn as_str(&self) -> &str {
        &self.0.0
    }
}
