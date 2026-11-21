#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub enum RetractLiftEnforce {
    #[default]
    #[serde(rename = "All Surfaces")]
    AllSurfaces,
    #[serde(rename = "Top Only")]
    TopOnly,
    #[serde(rename = "Bottom Only")]
    BottomOnly,
    #[serde(rename = "Top and Bottom")]
    TopAndBottom,
}
