use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrintHostType {
    #[serde(rename = "prusalink")]
    PrusaLink,
    #[serde(rename = "prusaconnect")]
    PrusaConnect,
    #[default]
    #[serde(rename = "octoprint")]
    OctoPrint,
    #[serde(rename = "crealityprint")]
    CrealityPrint,
    #[serde(rename = "duet")]
    Duet,
    #[serde(rename = "flashair")]
    FlashAir,
    #[serde(rename = "astrobox")]
    AstroBox,
    #[serde(rename = "repetier")]
    Repetier,
    #[serde(rename = "mks")]
    Mks,
    #[serde(rename = "esp3d")]
    Esp3d,
    #[serde(rename = "obico")]
    Obico,
    #[serde(rename = "flashforge")]
    Flashforge,
    #[serde(rename = "simplyprint")]
    SimplyPrint,
    #[serde(rename = "elegoolink")]
    ElegooLink,
    #[serde(rename = "3dprinteros")]
    ThreeDPrinterOs,
    #[serde(rename = "moonraker")]
    Moonraker,
}

/// `BedExcludeVolumeMode` (`PrintConfig.hpp:642-646`, keys `PrintConfig.cpp:742-746`).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum BedExcludeVolumeMode {
    #[default]
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "toolhead_offset")]
    ToolheadOffset,
    #[serde(rename = "per_extruder")]
    PerExtruder,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthorizationType {
    #[default]
    #[serde(rename = "key")]
    Key,
    #[serde(rename = "user")]
    User,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum NozzleVolumeType {
    #[default]
    #[serde(rename = "Standard")]
    Standard,
    #[serde(rename = "High Flow")]
    HighFlow,
    // PrintConfig.hpp:618-623 / PrintConfig.cpp:724-731: stable serialized value.
    #[serde(rename = "Hybrid")]
    Hybrid = 2,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NozzleVolumeTypes(pub Vec<NozzleVolumeType>);
