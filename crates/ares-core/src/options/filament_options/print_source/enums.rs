use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum RawOverhangFanThreshold {
    #[serde(rename = "0%")]
    Percent0,
    #[serde(rename = "10%")]
    Percent10,
    #[serde(rename = "25%")]
    Percent25,
    #[serde(rename = "50%")]
    Percent50,
    #[serde(rename = "75%")]
    Percent75,
    #[default]
    #[serde(rename = "95%")]
    Percent95,
}
