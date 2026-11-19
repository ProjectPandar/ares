#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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
