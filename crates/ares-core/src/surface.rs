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
