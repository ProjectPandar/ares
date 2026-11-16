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
    Standard = 0,
    #[serde(rename = "High Flow")]
    HighFlow = 1,
    #[serde(rename = "Hybrid")]
    Hybrid = 2,
    // PrintConfig.hpp:619-630: integer 4 is reserved.
    #[serde(rename = "TPU High Flow")]
    TpuHighFlow = 3,
    #[serde(rename = "E3D High Flow")]
    E3dHighFlow = 5,
    #[serde(rename = "Extra High Flow")]
    ExtraHighFlow = 6,
}

impl NozzleVolumeType {
    // PrintConfig.cpp:724-732: shared config keys, independent of GUI choices.
    pub(crate) const TYPE_KEYS: &[&str] = &[
        "Standard",
        "High Flow",
        "Hybrid",
        "TPU High Flow",
        "E3D High Flow",
        "Extra High Flow",
    ];
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NozzleVolumeTypes(pub Vec<NozzleVolumeType>);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BeltRotationAxis {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "x")]
    X,
    #[serde(rename = "y")]
    Y,
    #[serde(rename = "z")]
    Z,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RemapAxis {
    #[serde(rename = "pos_x")]
    PosX,
    #[serde(rename = "pos_y")]
    PosY,
    #[serde(rename = "pos_z")]
    PosZ,
    #[serde(rename = "neg_x")]
    NegX,
    #[serde(rename = "neg_y")]
    NegY,
    #[serde(rename = "neg_z")]
    NegZ,
    #[serde(rename = "rev_x")]
    RevX,
    #[serde(rename = "rev_y")]
    RevY,
    #[serde(rename = "rev_z")]
    RevZ,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FanDirection {
    #[serde(rename = "undefine")]
    Undefined,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "both")]
    Both,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ImexVizTheme {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "deuteranopia")]
    Deuteranopia,
    #[serde(rename = "tritanopia")]
    Tritanopia,
    #[serde(rename = "high_contrast")]
    HighContrast,
}
