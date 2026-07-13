use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessBrimType {
    #[default]
    #[serde(rename = "auto_brim")]
    AutoBrim,
    #[serde(rename = "brim_ears")]
    BrimEars,
    #[serde(rename = "painted")]
    Painted,
    #[serde(rename = "outer_only")]
    OuterOnly,
    #[serde(rename = "inner_only")]
    InnerOnly,
    #[serde(rename = "outer_and_inner")]
    OuterAndInner,
    #[serde(rename = "no_brim")]
    NoBrim,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessInternalBridgeFilter {
    #[default]
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "limited")]
    Limited,
    #[serde(rename = "nofilter")]
    NoFilter,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessExtraBridgeLayer {
    #[default]
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "external_bridge_only")]
    ExternalBridgeOnly,
    #[serde(rename = "internal_bridge_only")]
    InternalBridgeOnly,
    #[serde(rename = "apply_to_all")]
    ApplyToAll,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessGapFillTarget {
    #[serde(rename = "everywhere")]
    Everywhere,
    #[serde(rename = "topbottom")]
    TopBottom,
    #[default]
    #[serde(rename = "nowhere")]
    Nowhere,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessPerimeterGenerator {
    #[serde(rename = "classic")]
    Classic,
    #[default]
    #[serde(rename = "arachne")]
    Arachne,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSeamPosition {
    #[serde(rename = "nearest")]
    Nearest,
    #[default]
    #[serde(rename = "aligned")]
    Aligned,
    #[serde(rename = "aligned_back")]
    AlignedBack,
    #[serde(rename = "back")]
    Back,
    #[serde(rename = "random")]
    Random,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSlicingMode {
    #[default]
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "even_odd")]
    EvenOdd,
    #[serde(rename = "close_holes")]
    CloseHoles,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSupportBasePattern {
    #[serde(rename = "rectilinear")]
    Rectilinear,
    #[serde(rename = "rectilinear-grid")]
    RectilinearGrid,
    #[serde(rename = "honeycomb")]
    Honeycomb,
    #[serde(rename = "lightning")]
    Lightning,
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "hollow")]
    Hollow,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSupportInterfacePattern {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "rectilinear")]
    Rectilinear,
    #[serde(rename = "concentric")]
    Concentric,
    #[serde(rename = "rectilinear_interlaced")]
    RectilinearInterlaced,
    #[serde(rename = "grid")]
    Grid,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSupportStyle {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "grid")]
    Grid,
    #[serde(rename = "snug")]
    Snug,
    #[serde(rename = "organic")]
    Organic,
    #[serde(rename = "tree_slim")]
    TreeSlim,
    #[serde(rename = "tree_strong")]
    TreeStrong,
    #[serde(rename = "tree_hybrid")]
    TreeHybrid,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessSupportType {
    #[default]
    #[serde(rename = "normal(auto)")]
    NormalAuto,
    #[serde(rename = "tree(auto)")]
    TreeAuto,
    #[serde(rename = "normal(manual)")]
    NormalManual,
    #[serde(rename = "tree(manual)")]
    TreeManual,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProcessInfillPattern {
    #[serde(rename = "monotonic")]
    Monotonic,
    #[serde(rename = "monotonicline")]
    MonotonicLine,
    #[default]
    #[serde(rename = "rectilinear")]
    Rectilinear,
    #[serde(rename = "alignedrectilinear")]
    AlignedRectilinear,
    #[serde(rename = "zigzag")]
    ZigZag,
    #[serde(rename = "crosszag")]
    CrossZag,
    #[serde(rename = "lockedzag")]
    LockedZag,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "grid")]
    Grid,
    #[serde(rename = "triangles")]
    Triangles,
    #[serde(rename = "tri-hexagon")]
    TriHexagon,
    #[serde(rename = "cubic")]
    Cubic,
    #[serde(rename = "adaptivecubic")]
    AdaptiveCubic,
    #[serde(rename = "quartercubic")]
    QuarterCubic,
    #[serde(rename = "supportcubic")]
    SupportCubic,
    #[serde(rename = "lightning")]
    Lightning,
    #[serde(rename = "honeycomb")]
    Honeycomb,
    #[serde(rename = "3dhoneycomb")]
    ThreeDHoneycomb,
    #[serde(rename = "lateral-honeycomb")]
    LateralHoneycomb,
    #[serde(rename = "lateral-lattice")]
    LateralLattice,
    #[serde(rename = "crosshatch")]
    CrossHatch,
    #[serde(rename = "tpmsd")]
    TpmsD,
    #[serde(rename = "tpmsfk")]
    TpmsFk,
    #[serde(rename = "gyroid")]
    Gyroid,
    #[serde(rename = "concentric")]
    Concentric,
    #[serde(rename = "hilbertcurve")]
    HilbertCurve,
    #[serde(rename = "archimedeanchords")]
    ArchimedeanChords,
    #[serde(rename = "octagramspiral")]
    OctagramSpiral,
}
