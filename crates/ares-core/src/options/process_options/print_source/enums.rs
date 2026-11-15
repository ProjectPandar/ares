use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessDraftShield {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "enabled")]
    Enabled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessPrintOrder {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "as_obj_list")]
    AsObjectList,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessPrintSequence {
    #[serde(rename = "by layer")]
    ByLayer,
    #[serde(rename = "by object")]
    ByObject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSkirtType {
    #[serde(rename = "combined")]
    Combined,
    #[serde(rename = "perobject")]
    PerObject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessTimelapseType {
    #[serde(rename = "0")]
    Traditional,
    #[serde(rename = "1")]
    Smooth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessWipeTowerWallType {
    #[serde(rename = "rectangle")]
    Rectangle,
    #[serde(rename = "cone")]
    Cone,
    #[serde(rename = "rib")]
    Rib,
}
