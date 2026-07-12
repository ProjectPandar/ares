use crate::{Point2, PrintPathRole, SliceError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    #[test]
    fn classifies_external_perimeter_role() {
        let role = ExtrusionRole::ExternalPerimeter;

        assert!(role.is_perimeter());
        assert!(role.is_external_perimeter());
        assert!(!role.is_internal_perimeter());
    }

    #[test]
    fn classifies_internal_perimeter_role() {
        let role = ExtrusionRole::Perimeter;

        assert!(role.is_perimeter());
        assert!(role.is_internal_perimeter());
        assert!(!role.is_external_perimeter());
    }

    #[test]
    fn classifies_bridge_roles() {
        assert!(ExtrusionRole::BridgeInfill.is_bridge());
        assert!(ExtrusionRole::InternalBridgeInfill.is_bridge());
        assert!(ExtrusionRole::OverhangPerimeter.is_bridge());
        assert!(!ExtrusionRole::InternalInfill.is_bridge());
    }

    #[test]
    fn classifies_top_solid_infill_role() {
        let role = ExtrusionRole::TopSolidInfill;

        assert!(role.is_infill());
        assert!(role.is_solid_infill());
        assert!(role.is_top_surface());
        assert!(!ExtrusionRole::Ironing.is_top_surface());
        assert!(!ExtrusionRole::GapFill.is_infill());
    }

    #[test]
    fn maps_print_path_roles_to_libslic3r_extrusion_roles() {
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::ExternalPerimeter),
            ExtrusionRole::ExternalPerimeter
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::InternalPerimeter),
            ExtrusionRole::Perimeter
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::SparseInfill),
            ExtrusionRole::InternalInfill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::SolidInfill),
            ExtrusionRole::SolidInfill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::BottomSurface),
            ExtrusionRole::BottomSurface
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::TopSolidInfill),
            ExtrusionRole::TopSolidInfill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::Bridge),
            ExtrusionRole::BridgeInfill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::InternalBridge),
            ExtrusionRole::InternalBridgeInfill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::GapFill),
            ExtrusionRole::GapFill
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::Skirt),
            ExtrusionRole::Skirt
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::Brim),
            ExtrusionRole::Brim
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::OverhangPerimeter),
            ExtrusionRole::OverhangPerimeter
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::SupportMaterial),
            ExtrusionRole::SupportMaterial
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::SupportMaterialInterface),
            ExtrusionRole::SupportMaterialInterface
        );
        assert_eq!(
            ExtrusionRole::from_print_path_role(PrintPathRole::Ironing),
            ExtrusionRole::Ironing
        );
    }

    #[test]
    fn rejects_empty_extrusion_path() {
        assert_eq!(
            ExtrusionPath::new(ExtrusionRole::Perimeter, Vec::new()),
            Err(SliceError::InvalidInput(
                "extrusion path requires at least one point".to_owned()
            ))
        );
    }

    #[test]
    fn collection_from_paths_preserves_entity_order_and_role() {
        let first = ExtrusionPath::new(
            ExtrusionRole::ExternalPerimeter,
            vec![point(0.0, 0.0), point(1.0, 0.0)],
        )
        .unwrap();
        let second = ExtrusionPath::new(
            ExtrusionRole::InternalInfill,
            vec![point(0.0, 1.0), point(1.0, 1.0)],
        )
        .unwrap();

        let collection = ExtrusionEntityCollection::from_paths(vec![first, second]);

        assert_eq!(collection.len(), 2);
        assert_eq!(
            collection.paths()[0].role(),
            ExtrusionRole::ExternalPerimeter
        );
        assert_eq!(collection.paths()[1].role(), ExtrusionRole::InternalInfill);
    }
}
