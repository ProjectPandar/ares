use serde_json::Value;

use super::SliceOptions;
use crate::SliceError;

const KEY: &str = "enable_power_loss_recovery";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PowerLossRecoveryMode {
    PrinterConfiguration,
    Enable,
    Disable,
}

impl SliceOptions {
    pub(crate) fn power_loss_recovery_mode(&self) -> Result<PowerLossRecoveryMode, SliceError> {
        parse_mode(self.values().get(KEY))
    }
}

fn parse_mode(value: Option<&Value>) -> Result<PowerLossRecoveryMode, SliceError> {
    let Some(value) = value else {
        return Ok(PowerLossRecoveryMode::PrinterConfiguration);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(format!("{KEY} must be a string")));
    };
    match value {
        "printer_configuration" => Ok(PowerLossRecoveryMode::PrinterConfiguration),
        "enable" => Ok(PowerLossRecoveryMode::Enable),
        "disable" => Ok(PowerLossRecoveryMode::Disable),
        _ => Err(SliceError::InvalidInput(format!(
            "{KEY} must be one of printer_configuration, enable, disable"
        ))),
    }
}
