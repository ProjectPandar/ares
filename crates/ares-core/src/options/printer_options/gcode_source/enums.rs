use serde::{Deserialize, Serialize};

use crate::options::{Nullable, OrcaInt};

macro_rules! enum_vector {
    ($name:ident, $item:ty) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<$item>);
    };
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum NozzleType {
    #[default]
    #[serde(rename = "undefine")]
    Undefine,
    #[serde(rename = "hardened_steel")]
    HardenedSteel,
    #[serde(rename = "stainless_steel")]
    StainlessSteel,
    #[serde(rename = "tungsten_carbide")]
    TungstenCarbide,
    #[serde(rename = "brass")]
    Brass,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrinterStructure {
    #[default]
    #[serde(rename = "undefine")]
    Undefine,
    #[serde(rename = "corexy")]
    CoreXy,
    #[serde(rename = "i3")]
    I3,
    #[serde(rename = "hbot")]
    Hbot,
    #[serde(rename = "delta")]
    Delta,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ZHopType {
    #[serde(rename = "Auto Lift")]
    Auto,
    #[serde(rename = "Normal Lift")]
    Normal,
    #[default]
    #[serde(rename = "Slope Lift")]
    Slope,
    #[serde(rename = "Spiral Lift")]
    Spiral,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ExtruderType {
    #[default]
    #[serde(rename = "Direct Drive")]
    DirectDrive,
    #[serde(rename = "Bowden")]
    Bowden,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum WipeTowerType {
    #[serde(rename = "type1")]
    Type1,
    #[default]
    #[serde(rename = "type2")]
    Type2,
}

enum_vector!(NullableInts, Nullable<OrcaInt>);
enum_vector!(NullableNozzleTypes, Nullable<NozzleType>);
enum_vector!(ExtruderTypes, ExtruderType);
enum_vector!(RetractLiftEnforces, super::super::super::RetractLiftEnforce);
enum_vector!(ZHopTypes, ZHopType);
