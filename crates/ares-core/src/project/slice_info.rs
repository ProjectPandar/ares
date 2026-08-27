use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename = "config")]
pub(crate) struct SliceInfo {
    pub header: SliceInfoHeader,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SliceInfoHeader {
    #[serde(rename = "header_item", default)]
    pub items: Vec<SliceInfoHeaderItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SliceInfoHeaderItem {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
}
