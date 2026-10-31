use crate::{Point2, PrintPathRole, SliceError};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
pub enum ExtrusionRole {
    None,
    Perimeter,
    ExternalPerimeter,
    OverhangPerimeter,
    InternalInfill,
    SolidInfill,
    TopSolidInfill,
    BottomSurface,
    Ironing,
    BridgeInfill,
    InternalBridgeInfill,
    GapFill,
    Skirt,
    Brim,
    SupportMaterial,
    SupportMaterialInterface,
    SupportTransition,
    WipeTower,
    Custom,
    Mixed,
}

impl ExtrusionRole {
    pub const fn from_print_path_role(role: PrintPathRole) -> Self {
        match role {
            PrintPathRole::Skirt => Self::Skirt,
            PrintPathRole::Brim => Self::Brim,
            PrintPathRole::Bridge => Self::BridgeInfill,
            PrintPathRole::InternalBridge => Self::InternalBridgeInfill,
            PrintPathRole::GapFill => Self::GapFill,
            PrintPathRole::ExternalPerimeter => Self::ExternalPerimeter,
            PrintPathRole::OverhangPerimeter => Self::OverhangPerimeter,
            PrintPathRole::InternalPerimeter => Self::Perimeter,
            PrintPathRole::SparseInfill => Self::InternalInfill,
            PrintPathRole::SolidInfill => Self::SolidInfill,
            PrintPathRole::TopSolidInfill => Self::TopSolidInfill,
            PrintPathRole::BottomSurface => Self::BottomSurface,
            PrintPathRole::SupportMaterial => Self::SupportMaterial,
            PrintPathRole::SupportMaterialInterface => Self::SupportMaterialInterface,
            PrintPathRole::Ironing => Self::Ironing,
        }
    }

    /// `ExtrusionEntity::string_to_role` (`ExtrusionEntity.cpp:611-652`).
    pub fn from_name(name: &str) -> Self {
        match name {
            "Inner wall" => Self::Perimeter,
            "Outer wall" => Self::ExternalPerimeter,
            "Overhang wall" => Self::OverhangPerimeter,
            "Sparse infill" => Self::InternalInfill,
            "Internal solid infill" => Self::SolidInfill,
            "Top surface" => Self::TopSolidInfill,
            "Bottom surface" => Self::BottomSurface,
            "Ironing" => Self::Ironing,
            "Bridge" => Self::BridgeInfill,
            "Internal Bridge" => Self::InternalBridgeInfill,
            "Gap infill" => Self::GapFill,
            "Skirt" => Self::Skirt,
            "Brim" => Self::Brim,
            "Support" => Self::SupportMaterial,
            "Support interface" => Self::SupportMaterialInterface,
            "Support transition" => Self::SupportTransition,
            "Prime tower" => Self::WipeTower,
            "Custom" => Self::Custom,
            "Multiple" => Self::Mixed,
            _ => Self::None,
        }
    }

    pub const fn is_perimeter(self) -> bool {
        matches!(
            self,
            Self::Perimeter | Self::ExternalPerimeter | Self::OverhangPerimeter
        )
    }

    pub const fn is_internal_perimeter(self) -> bool {
        matches!(self, Self::Perimeter)
    }

    pub const fn is_external_perimeter(self) -> bool {
        matches!(self, Self::ExternalPerimeter)
    }

    pub const fn is_infill(self) -> bool {
        matches!(
            self,
            Self::InternalInfill
                | Self::SolidInfill
                | Self::TopSolidInfill
                | Self::BottomSurface
                | Self::Ironing
                | Self::BridgeInfill
                | Self::InternalBridgeInfill
        )
    }

    pub const fn is_top_surface(self) -> bool {
        matches!(self, Self::TopSolidInfill)
    }

    pub const fn is_solid_infill(self) -> bool {
        matches!(
            self,
            Self::BridgeInfill
                | Self::InternalBridgeInfill
                | Self::SolidInfill
                | Self::TopSolidInfill
                | Self::BottomSurface
                | Self::Ironing
        )
    }

    pub const fn is_bridge(self) -> bool {
        matches!(
            self,
            Self::BridgeInfill | Self::InternalBridgeInfill | Self::OverhangPerimeter
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtrusionPath {
    role: ExtrusionRole,
    points: Vec<Point2>,
}

impl ExtrusionPath {
    pub fn new(role: ExtrusionRole, points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.is_empty() {
            return Err(SliceError::InvalidInput(
                "extrusion path requires at least one point".to_owned(),
            ));
        }
        Ok(Self { role, points })
    }

    pub const fn role(&self) -> ExtrusionRole {
        self.role
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtrusionEntityCollection {
    paths: Vec<ExtrusionPath>,
}

impl ExtrusionEntityCollection {
    pub fn from_paths(paths: Vec<ExtrusionPath>) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &[ExtrusionPath] {
        &self.paths
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}
