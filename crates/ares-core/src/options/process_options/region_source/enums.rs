use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessEnsureVerticalShellThickness {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ensure_critical_only")]
    CriticalOnly,
    #[serde(rename = "ensure_moderate")]
    Moderate,
    #[serde(rename = "ensure_all")]
    EnsureAll,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessFuzzySkinType {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "external")]
    External,
    #[serde(rename = "hole")]
    Hole,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "allwalls")]
    AllWalls,
    #[serde(rename = "disabled_fuzzy")]
    Disabled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessNoiseType {
    #[serde(rename = "classic")]
    Classic,
    #[serde(rename = "perlin")]
    Perlin,
    #[serde(rename = "billow")]
    Billow,
    #[serde(rename = "ridgedmulti")]
    RidgedMulti,
    #[serde(rename = "voronoi")]
    Voronoi,
    #[serde(rename = "ripple")]
    Ripple,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessFuzzySkinMode {
    #[serde(rename = "displacement")]
    Displacement,
    #[serde(rename = "extrusion")]
    Extrusion,
    #[serde(rename = "combined")]
    Combined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessIroningType {
    #[serde(rename = "no ironing")]
    NoIroning,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "topmost")]
    Topmost,
    #[serde(rename = "solid")]
    Solid,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessCounterboreHoleBridging {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "partiallybridge")]
    PartiallyBridged,
    #[serde(rename = "sacrificiallayer")]
    SacrificialLayer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessTopSurfaceExpansionDirection {
    #[serde(rename = "inward_and_outward")]
    InwardAndOutward,
    #[serde(rename = "inward")]
    Inward,
    #[serde(rename = "outward")]
    Outward,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessWallSequence {
    #[serde(rename = "inner wall/outer wall")]
    InnerOuter,
    #[serde(rename = "outer wall/inner wall")]
    OuterInner,
    #[serde(rename = "inner-outer-inner wall")]
    InnerOuterInner,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessWallDirection {
    #[default]
    #[serde(rename = "ccw")]
    CounterClockwise,
    #[serde(rename = "cw")]
    Clockwise,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSeamScarfType {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "external")]
    External,
    #[serde(rename = "all")]
    All,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSurfaceFillOrder {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "outward")]
    Outward,
    #[serde(rename = "inward")]
    Inward,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessCenterOfSurfacePattern {
    #[serde(rename = "each_surface")]
    EachSurface,
    #[serde(rename = "each_model")]
    EachModel,
    #[serde(rename = "each_assembly")]
    EachAssembly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessAdaptiveInfillMode {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "distance_warp")]
    DistanceWarp,
    #[serde(rename = "smooth_blend")]
    SmoothBlend,
    #[serde(rename = "stepped_shells")]
    SteppedShells,
    #[serde(rename = "lobes")]
    Lobes,
    #[serde(rename = "normal_z")]
    NormalZ,
    #[serde(rename = "normal_y")]
    NormalY,
    #[serde(rename = "normal_x")]
    NormalX,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessAdaptiveInfillGradient {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "quadratic")]
    Quadratic,
    #[serde(rename = "exponential")]
    Exponential,
}
