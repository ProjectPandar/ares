use crate::Contour;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceType {
    Top,
    Bottom,
    BottomBridge,
    InternalAfterExternalBridge,
    Internal,
    InternalSolid,
    InternalBridge,
    SecondInternalBridge,
    InternalVoid,
    Perimeter,
}

impl SurfaceType {
    pub const fn is_top(self) -> bool {
        matches!(self, Self::Top)
    }

    pub const fn is_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomBridge)
    }

    pub const fn is_bridge(self) -> bool {
        matches!(self, Self::BottomBridge | Self::InternalBridge)
    }

    pub const fn is_internal_bridge(self) -> bool {
        matches!(self, Self::InternalBridge)
    }

    pub const fn is_external(self) -> bool {
        self.is_top() || self.is_bottom()
    }

    pub const fn is_internal(self) -> bool {
        !self.is_external()
    }

    pub const fn is_solid(self) -> bool {
        self.is_external() || matches!(self, Self::InternalSolid | Self::InternalBridge)
    }

    pub const fn is_solid_infill(self) -> bool {
        matches!(self, Self::InternalSolid)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    surface_type: SurfaceType,
    contour: Contour,
    thickness: f64,
    thickness_layers: u16,
    bridge_angle: f64,
    extra_perimeters: u16,
}

impl Surface {
    pub fn new(surface_type: SurfaceType, contour: Contour) -> Self {
        Self {
            surface_type,
            contour,
            thickness: -1.0,
            thickness_layers: 1,
            bridge_angle: -1.0,
            extra_perimeters: 0,
        }
    }

    pub const fn surface_type(&self) -> SurfaceType {
        self.surface_type
    }

    pub const fn thickness(&self) -> f64 {
        self.thickness
    }

    pub const fn thickness_layers(&self) -> u16 {
        self.thickness_layers
    }

    pub const fn bridge_angle(&self) -> f64 {
        self.bridge_angle
    }

    pub const fn extra_perimeters(&self) -> u16 {
        self.extra_perimeters
    }

    pub fn contour(&self) -> &Contour {
        &self.contour
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2;

    #[test]
    fn top_surface_is_top_external_and_solid() {
        let surface_type = SurfaceType::Top;

        assert!(surface_type.is_top());
        assert!(surface_type.is_external());
        assert!(surface_type.is_solid());
        assert!(!surface_type.is_bottom());
        assert!(!surface_type.is_bridge());
        assert!(!surface_type.is_internal());
    }

    #[test]
    fn bottom_bridge_surface_is_bottom_bridge_external_and_solid() {
        let surface_type = SurfaceType::BottomBridge;

        assert!(surface_type.is_bottom());
        assert!(surface_type.is_bridge());
        assert!(surface_type.is_external());
        assert!(surface_type.is_solid());
        assert!(!surface_type.is_top());
        assert!(!surface_type.is_internal_bridge());
    }

    #[test]
    fn internal_bridge_surface_is_bridge_internal_solid_and_internal_bridge() {
        let surface_type = SurfaceType::InternalBridge;

        assert!(surface_type.is_bridge());
        assert!(surface_type.is_internal());
        assert!(surface_type.is_solid());
        assert!(surface_type.is_internal_bridge());
        assert!(!surface_type.is_external());
        assert!(!surface_type.is_solid_infill());
    }

    #[test]
    fn internal_surface_is_internal_and_not_solid() {
        let surface_type = SurfaceType::Internal;

        assert!(surface_type.is_internal());
        assert!(!surface_type.is_solid());
        assert!(!surface_type.is_external());
        assert!(!surface_type.is_bridge());
    }

    #[test]
    fn new_surface_preserves_contour_and_uses_libslic3r_metadata_defaults() {
        let contour = triangle_contour();
        let surface = Surface::new(SurfaceType::Internal, contour.clone());

        assert_eq!(surface.surface_type(), SurfaceType::Internal);
        assert_eq!(surface.contour(), &contour);
        assert_eq!(surface.thickness(), -1.0);
        assert_eq!(surface.thickness_layers(), 1);
        assert_eq!(surface.bridge_angle(), -1.0);
        assert_eq!(surface.extra_perimeters(), 0);
    }

    fn triangle_contour() -> Contour {
        Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 1.0),
        ])
    }
}
