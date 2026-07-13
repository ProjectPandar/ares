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
