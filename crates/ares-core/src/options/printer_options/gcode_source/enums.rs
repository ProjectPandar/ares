use serde::{Deserialize, Serialize};

use crate::options::{Nullable, OrcaInt, config_types::semantic};

macro_rules! enum_vector {
    ($name:ident, $item:ty) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<$item>);
    };
}

macro_rules! nullable_enum_vector {
    ($name:ident, $item:ty) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<Nullable<$item>>);

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                semantic::serialize_nullable_vector(&self.0, serializer)
            }
        }
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

nullable_enum_vector!(NullableInts, OrcaInt);
nullable_enum_vector!(NullableNozzleTypes, NozzleType);
enum_vector!(ExtruderTypes, ExtruderType);
enum_vector!(RetractLiftEnforces, super::super::super::RetractLiftEnforce);
enum_vector!(ZHopTypes, ZHopType);
