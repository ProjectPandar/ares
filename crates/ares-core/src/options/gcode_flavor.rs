use super::SliceOptions;
use crate::SliceError;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub enum GCodeFlavor {
    #[default]
    #[serde(rename = "marlin")]
    MarlinLegacy,
    #[serde(rename = "klipper")]
    Klipper,
    #[serde(rename = "reprapfirmware")]
    RepRapFirmware,
    #[serde(rename = "repetier")]
    Repetier,
    #[serde(rename = "marlin2")]
    MarlinFirmware,
    #[serde(rename = "reprap")]
    RepRapSprinter,
    #[serde(rename = "teacup")]
    Teacup,
    #[serde(rename = "makerware")]
    MakerWare,
    #[serde(rename = "sailfish")]
    Sailfish,
    #[serde(rename = "mach3")]
    Mach3,
    #[serde(rename = "machinekit")]
    Machinekit,
    #[serde(rename = "smoothie")]
    Smoothie,
    #[serde(rename = "no-extrusion")]
    NoExtrusion,
}

impl GCodeFlavor {
    pub(crate) const fn supports_separate_travel_acceleration(self) -> bool {
        matches!(
            self,
            Self::Repetier | Self::MarlinFirmware | Self::RepRapFirmware
        )
    }

    pub(crate) fn emits_extrusion_axis_mode(self) -> bool {
        matches!(
            self,
            Self::RepRapSprinter
                | Self::RepRapFirmware
                | Self::MarlinLegacy
                | Self::MarlinFirmware
                | Self::Teacup
                | Self::Repetier
                | Self::Smoothie
                | Self::Klipper
        )
    }

    pub(crate) fn resets_absolute_e(self) -> bool {
        !matches!(self, Self::Mach3 | Self::MakerWare | Self::Sailfish)
    }

    pub(crate) fn skips_waiting_nozzle_temperature(self) -> bool {
        matches!(self, Self::MakerWare | Self::Sailfish)
    }

    pub(crate) fn nozzle_temperature_code(self, wait: bool) -> &'static str {
        match (self, wait) {
            (Self::RepRapFirmware, _) => "G10",
            (Self::Teacup, true) => "M104",
            (_, true) => "M109",
            _ => "M104",
        }
    }

    pub(crate) fn waits_after_nozzle_temperature(self, wait: bool) -> bool {
        wait && matches!(self, Self::Teacup | Self::RepRapFirmware)
    }
}

impl SliceOptions {
    pub(crate) fn gcode_flavor(&self) -> Result<GCodeFlavor, SliceError> {
        let Some(value) = self.values().get("gcode_flavor") else {
            return Ok(GCodeFlavor::MarlinLegacy);
        };
        let Some(value) = value.as_str() else {
            return Err(SliceError::InvalidInput(
                "gcode_flavor must be a string".to_owned(),
            ));
        };
        parse_active_gcode_flavor(value)
    }
}

fn parse_active_gcode_flavor(value: &str) -> Result<GCodeFlavor, SliceError> {
    match value {
        "marlin" => Ok(GCodeFlavor::MarlinLegacy),
        "klipper" => Ok(GCodeFlavor::Klipper),
        "reprapfirmware" => Ok(GCodeFlavor::RepRapFirmware),
        "repetier" => Ok(GCodeFlavor::Repetier),
        "marlin2" => Ok(GCodeFlavor::MarlinFirmware),
        _ => Err(SliceError::InvalidInput(format!("invalid value {value}"))),
    }
}
