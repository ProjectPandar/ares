use serde::{Deserialize, Serialize};

use super::super::super::config_types::{Nullable, OrcaFloat};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct NullableFloats(pub Vec<Nullable<OrcaFloat>>);

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ExtruderVariantLists(pub Vec<String>);

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
