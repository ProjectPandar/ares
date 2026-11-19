#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PowerLossRecoveryMode {
    #[default]
    #[serde(rename = "printer_configuration")]
    PrinterConfiguration,
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
}
