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
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NozzleVolumeTypes(pub Vec<NozzleVolumeType>);
