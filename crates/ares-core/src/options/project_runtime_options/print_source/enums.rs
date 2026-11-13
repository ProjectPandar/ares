use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProjectBedType {
    #[serde(rename = "Default Plate")]
    DefaultPlate,
    #[serde(rename = "Supertack Plate")]
    SupertackPlate,
    #[default]
    #[serde(rename = "Cool Plate")]
    CoolPlate,
    #[serde(rename = "Engineering Plate")]
    EngineeringPlate,
    #[serde(rename = "High Temp Plate")]
    HighTempPlate,
    #[serde(rename = "Textured PEI Plate")]
    TexturedPeiPlate,
    #[serde(rename = "Textured Cool Plate")]
    TexturedCoolPlate,
}

impl ProjectBedType {
    /// Effective selection after `PartPlate::set_bed_type` / `get_bed_type(true)`.
    #[must_use]
    pub const fn resolve(self, project_type: Self) -> Self {
        if matches!(self, Self::DefaultPlate) {
            project_type
        } else {
            self
        }
    }

    /// The registered `curr_bed_type` enum string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DefaultPlate => "Default Plate",
            Self::SupertackPlate => "Supertack Plate",
            Self::CoolPlate => "Cool Plate",
            Self::EngineeringPlate => "Engineering Plate",
            Self::HighTempPlate => "High Temp Plate",
            Self::TexturedPeiPlate => "Textured PEI Plate",
            Self::TexturedCoolPlate => "Textured Cool Plate",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProjectPrimeVolumeMode {
    #[default]
    Default,
    Saving,
    Fast,
}
