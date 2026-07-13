use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProjectFilamentMapMode {
    #[default]
    #[serde(rename = "Auto For Flush")]
    AutoForFlush,
    #[serde(rename = "Auto For Match")]
    AutoForMatch,
    #[serde(rename = "Manual")]
    Manual,
}
