use serde::{Deserialize, Serialize};

use super::super::super::{
    ProjectBedType,
    config_types::{Nullable, OrcaFloat, semantic},
};

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

impl DefaultBedType {
    /// `Preset::get_default_bed_type` (Preset.cpp:1140–1175), after inheritance resolution.
    #[must_use]
    pub fn resolve(&self, model_id: &str) -> ProjectBedType {
        Self::resolve_hint(&self.0, model_id)
    }

    pub(crate) fn resolve_hint(hint: &str, model_id: &str) -> ProjectBedType {
        use ProjectBedType::{
            CoolPlate, EngineeringPlate, HighTempPlate, SupertackPlate, TexturedCoolPlate,
            TexturedPeiPlate,
        };
        if hint.is_empty() {
            return if matches!(model_id, "BL-P001" | "BL-P002" | "C13") {
                CoolPlate
            } else {
                HighTempPlate
            };
        }
        match hint {
            "Cool Plate" => return CoolPlate,
            "Engineering Plate" => return EngineeringPlate,
            "High Temp Plate" => return HighTempPlate,
            "Textured PEI Plate" => return TexturedPeiPlate,
            "Textured Cool Plate" => return TexturedCoolPlate,
            "Supertack Plate" => return SupertackPlate,
            _ => {}
        }
        let text = hint.trim_start_matches(|c: char| matches!(c, ' ' | '\t'..='\r'));
        let value = text
            .strip_prefix('+')
            .unwrap_or(text)
            .bytes()
            .take_while(u8::is_ascii_digit)
            .fold(0_u32, |value, digit| {
                value
                    .saturating_mul(10)
                    .saturating_add(u32::from(digit - b'0'))
            });
        match value {
            1 => CoolPlate,
            2 => EngineeringPlate,
            3 => HighTempPlate,
            4 => TexturedPeiPlate,
            5 => TexturedCoolPlate,
            6 => SupertackPlate,
            _ => HighTempPlate,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ThumbnailDefinitions(pub super::super::super::config_types::OrcaString);

impl ThumbnailDefinitions {
    pub fn as_str(&self) -> &str {
        &self.0.0
    }
}
